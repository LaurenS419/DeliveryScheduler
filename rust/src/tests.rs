use super::*;

/// Creates mock Hospitals.
///
/// # Returns
/// A vector of `Hospital` structs representing the mock hospitals.
fn mock_hospitals() -> Vec<Hospital> {
    vec![
        Hospital {
            name: "Nest".to_string(),
            north_m: 0,
            east_m: 0,
        },
        Hospital {
            name: "Bravo".to_string(),
            north_m: 1000,
            east_m: 0,
        },
        Hospital {
            name: "Charlie".to_string(),
            north_m: 50000,
            east_m: 0,
        },
        Hospital {
            name: "Delta".to_string(),
            north_m: -50000,
            east_m: -50000,
        },
    ]
}

/// Creates a mock DroneScheduler.
///
/// # Returns
/// A DroneScheduler struct with false values.
fn mock_scheduler() -> DroneScheduler {
    DroneScheduler {
        hospitals: mock_hospitals(),
        num_drones: 2,
        max_packages_per_drone: 3,
        drone_speed_mps: 30,
        drone_max_cumulative_range_m: 160 * 1000,
        emergency_orders: VecDeque::new(),
        close_resupply_orders: VecDeque::new(),
        far_resupply_orders: VecDeque::new(),
        num_orders_completed: 0,
        completed_deliveries: vec![],
        num_reserve_drones: 1,
        drones: vec![
            Drone {
                id: 0,
                status: DroneStatus::Idle,
            },
            Drone {
                id: 1,
                status: DroneStatus::Idle,
            },
        ],
    }
}

/// Creates a mock DroneScheduler with orders.
///
/// # Returns
/// A DroneScheduler struct with false values and orders.
fn mock_scheduler_with_orders() -> DroneScheduler {
    let mut scheduler = DroneScheduler {
        hospitals: mock_hospitals(),
        num_drones: 2,
        max_packages_per_drone: 3,
        drone_speed_mps: 30,
        drone_max_cumulative_range_m: 160_000,
        emergency_orders: VecDeque::new(),
        close_resupply_orders: VecDeque::new(),
        far_resupply_orders: VecDeque::new(),
        num_orders_completed: 0,
        completed_deliveries: vec![],
        num_reserve_drones: 1,
        drones: vec![
            Drone {
                id: 0,
                status: DroneStatus::Idle,
            },
            Drone {
                id: 1,
                status: DroneStatus::Idle,
            },
        ],
    };

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Resupply,
    });

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Charlie".to_string(),
        priority: OrderPriority::Resupply,
    });

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Emergency,
    });

    scheduler
}

/// Creates a mock DroneScheduler with orders.
///
/// # Returns
/// A DroneScheduler struct with false values and orders.
fn mock_scheduler_with_orders_no_emergency() -> DroneScheduler {
    let mut scheduler = DroneScheduler {
        hospitals: mock_hospitals(),
        num_drones: 2,
        max_packages_per_drone: 3,
        drone_speed_mps: 30,
        drone_max_cumulative_range_m: 160_000,
        emergency_orders: VecDeque::new(),
        close_resupply_orders: VecDeque::new(),
        far_resupply_orders: VecDeque::new(),
        num_orders_completed: 0,
        completed_deliveries: vec![],
        num_reserve_drones: 1,
        drones: vec![
            Drone {
                id: 0,
                status: DroneStatus::Idle,
            },
            Drone {
                id: 1,
                status: DroneStatus::Idle,
            },
        ],
    };

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Resupply,
    });

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Charlie".to_string(),
        priority: OrderPriority::Resupply,
    });

    scheduler
}

// -- Tests -- //

// dist tests
#[test]
fn test_dist() {
    assert_eq!((dist((0, 0), (3, 4))), 5.0);
}

#[test]
fn test_dist_negatives() {
    assert_eq!((dist((0, 0), (-3, -4))), 5.0);
}

#[test]
fn test_dist_same() {
    assert_eq!((dist((10, 10), (10, 10))), 0.0);
}

// Drone tests

#[test]
fn test_mark_idle_if_returned() {
    let mut drone = Drone {
        id: 0,
        status: DroneStatus::InFlight {
            launch_time: 0,
            return_time: 100,
            orders: vec![],
        },
    };
    drone.mark_idle_if_returned(101);
    match drone.status {
        DroneStatus::Idle => {}
        _ => panic!("Drone should be idle"),
    }
}

#[test]
fn test_not_mark_idle_if_not_returned() {
    let mut drone = Drone {
        id: 0,
        status: DroneStatus::InFlight {
            launch_time: 0,
            return_time: 100,
            orders: vec![],
        },
    };
    drone.mark_idle_if_returned(99);
    match drone.status {
        DroneStatus::InFlight { .. } => {}
        _ => panic!("Drone should be InFlight"),
    }
}

// DroneScheduler tests

// queue_order

#[test]
fn test_queue_order_emergency() {
    let mut scheduler = mock_scheduler();
    let order = Order {
        time: 10,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Emergency,
    };
    scheduler.queue_order(order.clone());
    assert_eq!(scheduler.emergency_orders.len(), 1);
    assert_eq!(scheduler.emergency_orders[0].hospital, "Bravo");
}

#[test]
fn test_queue_order_resupply_close() {
    let mut scheduler = mock_scheduler();
    let order = Order {
        time: 20,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Resupply,
    };
    scheduler.queue_order(order.clone());
    assert_eq!(scheduler.close_resupply_orders.len(), 1);
}

#[test]
fn test_queue_order_resupply_far() {
    let mut scheduler = mock_scheduler();
    let order = Order {
        time: 30,
        hospital: "Charlie".to_string(),
        priority: OrderPriority::Resupply,
    };
    scheduler.queue_order(order.clone());
    assert_eq!(scheduler.far_resupply_orders.len(), 1);
}

#[test]
fn test_queue_order_fifo() {
    let mut scheduler = mock_scheduler();

    let orders = vec![
        Order {
            time: 20,
            hospital: "Bravo".to_string(),
            priority: OrderPriority::Resupply,
        },
        Order {
            time: 30,
            hospital: "Bravo".to_string(),
            priority: OrderPriority::Resupply,
        },
    ];

    scheduler.queue_order(orders[0].clone());
    scheduler.queue_order(orders[1].clone());
    assert_eq!(scheduler.close_resupply_orders.len(), 2);
    assert_eq!(scheduler.close_resupply_orders.front().unwrap().time, 20);
}

// launch_flights

#[test]
fn test_launch_flights() {
    let mut scheduler = mock_scheduler();
    scheduler.queue_order(Order {
        time: 0,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Emergency,
    });
    let flights = scheduler.launch_flights(60);
    assert_eq!(flights.len(), 1);
    assert_eq!(flights[0].orders.len(), 1);
}

// get_hospital

#[test]
fn test_get_hospital_found() {
    let scheduler = mock_scheduler();

    let h = scheduler.get_hospital("Bravo");
    assert!(h.is_some());
    let h = h.unwrap();
    assert_eq!(h.north_m, 1000);
    assert_eq!(h.east_m, 0);
}

#[test]
fn test_get_hospital_not_found() {
    let scheduler = mock_scheduler();

    let h = scheduler.get_hospital("NonExistentHospital");
    assert!(h.is_none());
}

// process_orders_for_flight

#[test]
fn test_process_orders_for_flight() {
    let scheduler = mock_scheduler();

    let orders = vec![
        Order {
            time: 0,
            hospital: "Bravo".to_string(),
            priority: OrderPriority::Resupply,
        },
        Order {
            time: 0,
            hospital: "Charlie".to_string(),
            priority: OrderPriority::Resupply,
        },
    ];

    let (records, return_time) = scheduler.process_orders_for_flight(&orders, 60);

    assert_eq!(records.len(), 2);
    assert!(return_time > 60); // It must be greater than launch time
    assert_eq!(records[0].hospital, "Bravo");
    assert_eq!(records[1].hospital, "Charlie");
}

// prepare_orders_for_drone

#[test]
fn test_prepare_orders_for_drone_emergency_priority() {
    let mut scheduler = mock_scheduler_with_orders();

    let (orders, return_time, records) = scheduler.prepare_orders_for_drone(120, 2);

    assert!(!orders.is_empty());
    assert_eq!(orders[0].priority, OrderPriority::Emergency);
    assert!(return_time > 120);
    assert_eq!(records.len(), orders.len());
}

// get_nearest_order

#[test]
fn test_get_nearest_order_emergency() {
    let mut scheduler = mock_scheduler_with_orders();
    let reference_hospital = scheduler.get_hospital("Bravo").unwrap();

    if let Some((order, queue, _)) = scheduler.get_nearest_order(&reference_hospital) {
        assert!(matches!(queue, OrderQueue::Emergency)); // Emergency is prioritized
        assert_eq!(order.hospital, "Bravo");
    } else {
        panic!("Expected a nearest order, got none");
    }
}

#[test]
fn test_get_nearest_order_no_emergency() {
    let mut scheduler = mock_scheduler_with_orders_no_emergency();
    let reference_hospital = scheduler.get_hospital("Bravo").unwrap();

    if let Some((order, queue, _)) = scheduler.get_nearest_order(&reference_hospital) {
        assert!(matches!(queue, OrderQueue::CloseResupply)); // Bravo since the same hospital is the closest
        assert_eq!(order.hospital, "Bravo");
    } else {
        panic!("Expected a nearest order, got none");
    }
}

// try_nearest_neighbour

#[test]
fn test_try_nearest_neighbour() {
    let mut scheduler = mock_scheduler();

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Bravo".to_string(),
        priority: OrderPriority::Resupply,
    });

    let mut orders = vec![Order {
        time: 0,
        hospital: "Charlie".to_string(),
        priority: OrderPriority::Resupply,
    }];

    scheduler.try_nearest_neighbour(&mut orders);
    assert_eq!(orders.len(), 2);
}

#[test]
fn test_try_no_nearest_neighbour() {
    let mut scheduler = mock_scheduler();

    scheduler.queue_order(Order {
        time: 0,
        hospital: "Charlie".to_string(),
        priority: OrderPriority::Resupply,
    });

    let mut orders = vec![Order {
        time: 0,
        hospital: "Delta".to_string(),
        priority: OrderPriority::Resupply,
    }];

    scheduler.try_nearest_neighbour(&mut orders);
    assert_eq!(orders.len(), 1);
}
