use csv::ReaderBuilder;
use csv::Writer;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;
use std::error::Error;

/// The number of Drones at each nest.
const NUM_DRONES: usize = 10;

/// A Drone can carry between 1 and this many packages per flight.
/// Note: a Drone can deliver more than 1 package per stop.
const MAX_PACKAGES_PER_DRONE: usize = 3;

/// The (constant) ground speed all Drones fly at (in m/s).
const DRONE_SPEED_MPS: u32 = 30;

/// The farthest total roundtrip distance a Drone can fly (in m).
const DRONE_MAX_CUMULATIVE_RANGE_M: u32 = 160 * 1000; // 160 km -> meters

/// The number of Drones to keep on-hand for emergencies only to ensure we can launch emergency deliveries immediately/faster
const NUM_RESERVE_DRONES: usize = 3;

// The following structures describe the input data schema.
// You shouldn't need to modify them.

#[derive(Debug, Deserialize, Clone)]
struct Hospital {
    name: String,
    north_m: i32, // x
    east_m: i32,  // y
}

#[derive(Debug, Deserialize, Clone)]
struct Order {
    time: u32,
    hospital: String,
    priority: OrderPriority,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
enum OrderPriority {
    Emergency,
    Resupply,
}

/// Represents a launched flight.
/// Feel free to extend.
#[derive(Debug)]
struct Flight {
    launch_time: u32,
    orders: Vec<Order>,
}

/// Record of each delivery for metrics
#[derive(Debug, Serialize)]
struct DeliveryRecord {
    order_time: u32,
    hospital: String,
    priority: OrderPriority,
    launch_time: u32,
    delivered_time: u32,
}

/// Record for fulfilled vs unfulfilled orders
#[derive(Serialize)]
struct CountRecord {
    fulfilled_orders: usize,
    unfulfilled_orders: usize,
}

/// Represents a Drone and its current status
#[derive(Debug)]
struct Drone {
    id: usize,
    status: DroneStatus,
}

/// Status for Drones
#[derive(Debug)]
enum DroneStatus {
    Idle,
    InFlight {
        launch_time: u32,
        return_time: u32,
        orders: Vec<Order>,
    },
}

/// This is the component we're like you to implement.
struct DroneScheduler {
    hospitals: Vec<Hospital>,
    num_drones: usize,
    max_packages_per_drone: usize,
    drone_speed_mps: u32,
    drone_max_cumulative_range_m: u32,

    // Different queues for emergency orders and resupply orders
    emergency_orders: VecDeque<Order>,
    resupply_orders: VecDeque<Order>,

    num_orders_completed: usize, // number of orders completed for printing
    completed_deliveries: Vec<DeliveryRecord>, // vector of delivery records
    num_reserve_drones: usize,   // amount of Drones to keep for emergency deliveries only
    drones: Vec<Drone>,          // vector of Drones
}

impl Drone {
    /// Marks the Drone as Idle if its return_time has passed.
    ///
    /// This function checks whether a Drone that was previously in-flight
    /// has completed its journey based on the current time. If so, it updates its status
    /// to `Idle`, making it available for new deliveries.
    ///
    /// # Arguments
    /// * `current_time` - The current time in seconds since midnight.
    fn mark_idle_if_returned(&mut self, current_time: u32) {
        if let DroneStatus::InFlight { return_time, .. } = self.status {
            if return_time <= current_time {
                self.status = DroneStatus::Idle;
            }
        }
    }
}

impl DroneScheduler {
    /// Queues a new order based on its priority and distance from the Nest.
    ///
    /// Emergency orders are always prioritized and pushed to the `emergency_orders` queue.
    /// Resupply orders are put into the `resupply_orders` queue.
    ///
    /// Note: this function is called every time a new order arrives.
    ///
    /// # Arguments
    /// * `order` - The incoming order to queue.
    fn queue_order(&mut self, order: Order) {
        match order.priority {
            OrderPriority::Emergency => self.emergency_orders.push_back(order),
            OrderPriority::Resupply => self.resupply_orders.push_back(order),
        }
    }

    /// Returns a list of flights which should be launched right now for all available Drones.
    ///
    /// This method checks all Drones to see if any have returned and are now Idle.
    /// Then, for each Idle Drone, it prepares a list of Orders to serve in the flight, updates
    /// the Drone’s status, and logs the deliveries.
    ///
    /// Note: will be called periodically (approximately once a minute).
    ///
    /// # Arguments
    /// * `current_time` - The current time in seconds since midnight.
    ///
    /// # Returns
    /// A vector of `Flight` structs representing the launched flights.
    fn launch_flights(&mut self, current_time: u32) -> Vec<Flight> {
        let mut result = vec![];
        let mut drones_available = 0;

        // See if any Drones have returned since last launch_flights call
        for drone in &mut self.drones {
            drone.mark_idle_if_returned(current_time);

            // Count up Drones available for launch
            if let DroneStatus::Idle = drone.status {
                drones_available += 1;
            }
        }

        // Give orders to all the Idle Drones
        for i in 0..self.drones.len() {
            if let DroneStatus::InFlight { .. } = self.drones[i].status {
                continue;
            }

            let (orders, return_time, records) =
                self.prepare_orders_for_drone(current_time, drones_available);

            // If there are no orders to fulfill, don't send Drone
            if orders.is_empty() {
                continue;
            }

            // assign the Drone its new info
            self.drones[i].status = DroneStatus::InFlight {
                launch_time: current_time,
                return_time,
                orders: orders.clone(),
            };

            // Metrics
            drones_available -= 1; // 1 less drone available
            self.num_orders_completed += orders.len();
            self.completed_deliveries.extend(records);

            result.push(Flight {
                launch_time: current_time,
                orders: orders.clone(),
            });
        }

        result
    }

    /// Retrieves a `Hospital` struct given its name.
    ///
    /// This helper method searches the stored list of hospitals and returns a
    /// cloned copy of the matching one.
    ///
    /// # Arguments
    /// * `hospital_name` - Name of the hospital.
    ///
    /// # Returns
    /// An `Option` containing a `Hospital` if found, or `None`.
    fn get_hospital(&self, hospital_name: &str) -> Option<Hospital> {
        self.hospitals
            .iter()
            .find(|h| h.name == hospital_name)
            .cloned()
    }

    /// Processes the orders on a flight and returns delivery records and return time.
    ///
    /// Calculates travel times between hospitals, records when each order is delivered,
    /// and computes the full round-trip time for the Drone to return to the Nest.
    ///
    /// # Arguments
    /// * `orders` - The list of orders this Drone is fulfilling.
    /// * `launch_time` - The time at which the Drone was launched.
    ///
    /// # Returns
    /// A tuple containing:
    /// * A list of `DeliveryRecord`s with detailed delivery metadata.
    /// * The expected return time of the Drone.
    fn process_orders_for_flight(
        &self,
        orders: &[Order],
        launch_time: u32,
    ) -> (Vec<DeliveryRecord>, u32) {
        let mut records = Vec::with_capacity(orders.len()); // records for CSV
        let mut elapsed_secs = 0.0; // travel time
        let mut total_dist = 0.0; // distance travelled
        let mut prev = (0, 0); // coords of previous location

        for order in orders {
            let h = self.get_hospital(&order.hospital).unwrap();

            let next = (h.north_m, h.east_m); // next destination coords
            let segment_dist = dist(prev, next); // distance from prev destination to next

            total_dist += segment_dist; // sum the distance
            elapsed_secs += segment_dist / self.drone_speed_mps as f64; // add the travel time to it

            // Add to records
            records.push(DeliveryRecord {
                order_time: order.time,
                hospital: order.hospital.clone(),
                priority: order.priority.clone(),
                launch_time,
                delivered_time: launch_time + elapsed_secs.round() as u32,
            });

            prev = next; // Update the prev stop
        }

        let return_dist = dist(prev, (0, 0)); // Return from last hospital to Nest
        total_dist += return_dist;

        let return_time = launch_time + (total_dist / self.drone_speed_mps as f64).round() as u32;

        (records, return_time)
    }

    /// Orders are selected based on the following priority:
    /// 1. Emergency (always picked if available)
    /// 3. Resupply
    ///
    /// Uses a FIFO strategy to add 1–2 more orders to optimize route efficiency,
    /// provided the Drone has enough range to return safely.
    ///
    /// # Arguments
    /// * `current_time` - The current time in seconds since midnight.
    /// * `drones_available` - The number of Drones currently idle and available.
    ///
    /// # Returns
    /// A tuple of:
    /// * Orders the Drone will deliver.
    /// * The return time for the Drone.
    /// * A list of delivery records for logging and metrics.
    fn prepare_orders_for_drone(
        &mut self,
        current_time: u32,
        drones_available: usize,
    ) -> (Vec<Order>, u32, Vec<DeliveryRecord>) {
        let mut orders = vec![];

        if !self.emergency_orders.is_empty() {
            orders.push(self.emergency_orders.pop_front().unwrap());

            // Try to add deliveries
            self.try_next_order(&mut orders);
            self.try_next_order(&mut orders);
        } else if self.resupply_orders.len() > 1 && drones_available > self.num_reserve_drones {
            orders.push(self.resupply_orders.pop_front().unwrap());

            // Try to add deliveries
            self.try_next_order(&mut orders);
            self.try_next_order(&mut orders);
        }

        let (records, return_time) = self.process_orders_for_flight(&orders, current_time);

        (orders, return_time, records)
    }

    /// Attempts to add the next order in the queue to an existing order list.
    ///
    /// This function looks for the nearest queued order (from any queue) to the last
    /// hospital in the current order list. If adding that hospital to the route still
    /// keeps the total round-trip distance within the allowed range, the order is added
    /// and removed from its respective queue.
    ///
    /// # Arguments
    /// * `orders` - Mutable reference to the current list of selected orders.
    fn try_next_order(&mut self, orders: &mut Vec<Order>) {
        // gets the last order in the orders list and its hospital
        if let Some(last_order) = orders.last() {
            let from_hospital = self.get_hospital(&last_order.hospital).unwrap();

            // If there is a nearest order to it, find its hospital (prioritizing emergencies)
            let (next_order, queue, index) = if let Some(order) = self.emergency_orders.front() {
                (order, OrderPriority::Emergency, 0)
            } else if let Some(order) = self.resupply_orders.front() {
                (order, OrderPriority::Resupply, 0)
            } else {
                return;
            };

            let next_hosp = self.get_hospital(&next_order.hospital).unwrap();

            let mut total_distance = 0.0;

            // Distance from Nest to first hospital in orders
            let first_hosp = self.get_hospital(&orders[0].hospital).unwrap();
            total_distance += dist((0, 0), (first_hosp.north_m, first_hosp.east_m));

            // Distance between all consecutive hospitals in the orders
            for h in orders.windows(2) {
                let h1 = self.get_hospital(&h[0].hospital).unwrap();
                let h2 = self.get_hospital(&h[1].hospital).unwrap();
                total_distance += dist((h1.north_m, h1.east_m), (h2.north_m, h2.east_m));
            }

            // Distance from last hospital to new hospital
            total_distance += dist(
                (from_hospital.north_m, from_hospital.east_m),
                (next_hosp.north_m, next_hosp.east_m),
            );

            // Distance back to nest
            total_distance += dist((next_hosp.north_m, next_hosp.east_m), (0, 0));

            // Ensures the Drone can reach this nearest hospital
            if total_distance <= self.drone_max_cumulative_range_m as f64 {
                orders.push(next_order.clone());

                // Remove the order from its queue
                match queue {
                    OrderPriority::Emergency => {
                        self.emergency_orders.remove(index);
                    }
                    OrderPriority::Resupply => {
                        self.resupply_orders.remove(index);
                    }
                }
            }
        }
    }
}

/// Calculates the Euclidean distance between 2 points.
///
/// # Arguments
/// * `a` - Point with coordinates (north_m, east_m)
/// * `b` - Point with coordinates (north_m, east_m)
///
/// # Returns
/// Euclidean distance in metres between the 2 points
fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
    let dx = (a.0 - b.0) as f64;
    let dy = (a.1 - b.1) as f64;

    (dx * dx + dy * dy).sqrt()
}

/// Writes output metrics to `path`.
///
/// fulfilled_orders represents the amount of orders completed, and
/// unfulfilled_orders represents the amount of orders not delivered.
/// These are both written to `path` for later analysis.
///
/// # Arguments
/// * `completed` - number of completed orders
/// * `unfulfilled` - number of unfulfilled orders
/// * `path` - path to store output csv
fn write_order_counts(
    completed: usize,
    unfulfilled: usize,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    let count = CountRecord {
        fulfilled_orders: completed,
        unfulfilled_orders: unfulfilled,
    };
    writer.serialize(count)?;
    writer.flush()?;
    println!("Wrote order counts to {}", path);
    Ok(())
}

/// Writes order records to `path`.
///
/// Each order record has columns order_time, hospital, priority, launch_time, and delivered_time.
/// If the order was unfulfilled, launch_time = delivered_time = 0.
/// These are written to `path` for later analysis.
///
/// # Arguments
/// * `completed` - Vector of `DeliveryRecord`s of completed orders
/// * `emergency` - VecDeque of `Order`s with outstanding emergency orders
/// * `close_resupply` - VecDeque of `Order`s with outstanding close_resupply orders
/// * `far_resupply` - VecDeque of `Order`s with outstanding far_resupply orders
/// * `path` - path to store output csv
fn write_deliveries(
    completed: &[DeliveryRecord],
    emergency: &VecDeque<Order>,
    resupply: &VecDeque<Order>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;

    // Completed deliveries
    for rec in completed {
        writer.serialize(rec)?;
    }

    // Unfulfilled deliveries
    let unfulfilled_orders = emergency.iter().chain(resupply);

    for order in unfulfilled_orders {
        writer.serialize(DeliveryRecord {
            order_time: order.time,
            hospital: order.hospital.clone(),
            priority: order.priority.clone(),
            launch_time: 0,
            delivered_time: 0,
        })?;
    }

    writer.flush()?;
    println!(
        "Wrote {} delivery records to {}",
        completed.len() + emergency.len() + resupply.len(),
        path
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let hospitals: Result<Vec<_>, _> = ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_path("../inputs/hospitals.csv")?
        .deserialize()
        .collect();

    let mut scheduler = DroneScheduler {
        hospitals: hospitals?,
        num_drones: NUM_DRONES,
        max_packages_per_drone: MAX_PACKAGES_PER_DRONE,
        drone_speed_mps: DRONE_SPEED_MPS,
        drone_max_cumulative_range_m: DRONE_MAX_CUMULATIVE_RANGE_M,
        emergency_orders: VecDeque::new(),
        resupply_orders: VecDeque::new(),
        num_orders_completed: 0,
        completed_deliveries: Vec::new(),
        num_reserve_drones: NUM_RESERVE_DRONES,
        // create the Drones
        drones: (0..NUM_DRONES)
            .map(|id| Drone {
                id,
                status: DroneStatus::Idle,
            })
            .collect(),
    };

    let orders: Result<Vec<_>, _> = ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_path("../inputs/orders.csv")?
        .deserialize()
        .collect();
    let mut orders = orders?;
    orders.reverse();

    // Simulate time advancing forward until the end of the day.
    const SEC_PER_DAY: u32 = 24 * 60 * 60;
    let start_time = orders.last().map_or(0, |o: &Order| o.time);
    for current_time in start_time..SEC_PER_DAY {
        // Find and queue new orders.
        while let Some(order) = orders.last() {
            if order.time > current_time {
                break;
            }
            println!(
                "[{}] {:?} order received to {}",
                current_time, order.priority, order.hospital
            );
            scheduler.queue_order(orders.pop().unwrap());
        }

        // Once a minute, poke the flight launcher.
        if current_time % 60 == 0 {
            let flights = scheduler.launch_flights(current_time);
            if !flights.is_empty() {
                println!("[{}] Scheduling flights:", current_time);
                for flight in flights {
                    println!("\t{:?}", flight);
                }
            }
        }
    }

    println!(
        "{} unfulfilled orders at the end of the day",
        scheduler.emergency_orders.len() + scheduler.resupply_orders.len() //scheduler.unfulfilled_orders.len()
    );

    println!(
        "{} orders completed at the end of the day",
        scheduler.num_orders_completed
    );

    // Write metrics into output files
    write_order_counts(
        scheduler.num_orders_completed,
        scheduler.emergency_orders.len() + scheduler.resupply_orders.len(),
        "../output/count.csv",
    )?;

    write_deliveries(
        &scheduler.completed_deliveries,
        &scheduler.emergency_orders,
        &scheduler.resupply_orders,
        "../output/deliveries.csv",
    )?;

    Ok(())
}
