---
title: sysinfo 0.30 Documentation
description: A comprehensive guide to using sysinfo 0.30 in Rust
author: DataScienceBioLab
date: 2024-03-18
tags: [rust, crates, system-monitoring, documentation]
---

# sysinfo 0.30 Documentation Guide

## Overview

`sysinfo` is a Rust crate for retrieving system information across multiple platforms. This guide provides documentation for version 0.30, highlighting key features, API changes, and proper usage patterns.

## Supported Operating Systems

- Windows
- macOS
- Linux
- FreeBSD
- Android
- iOS
- Raspberry Pi

## Key Features

- CPU information and usage statistics
- Memory usage reporting
- Disk information and statistics
- Network interface monitoring
- Process information tracking
- System information querying

## Important Changes in 0.30

The 0.30 version of sysinfo introduced several significant API changes that differ from both older and newer versions:

1. **Thread Information**: `thread_count()` was replaced with `thread_kind()` which returns an `Option<ThreadKind>` enum instead of a count
2. **Disks API**: Changed how disks are accessed and refreshed
3. **Networks API**: Modified network refresh and access methods
4. **Data ownership**: Some data structures no longer implement `Clone`

## Basic Usage

### Initializing System Information

```rust
use sysinfo::{System, SystemExt};

// Create a new System with all information
let mut system = System::new_all();

// First refresh all information
system.refresh_all();

// Now you can access the information
println!("System name: {:?}", System::name());
println!("System kernel version: {:?}", System::kernel_version());
println!("System OS version: {:?}", System::os_version());
println!("System host name: {:?}", System::host_name());
```

### Working with Networks

Networks access has changed in 0.30. The recommended approach is to use `Networks::new_with_refreshed_list()` instead of accessing networks through the System struct:

```rust
use sysinfo::{Networks, NetworkExt};

// Create Networks instance with refreshed data
let networks = Networks::new_with_refreshed_list();

// Iterate through network interfaces
for (interface_name, network) in &networks {
    println!(
        "{}: {} B received / {} B transmitted",
        interface_name,
        network.received(),
        network.transmitted()
    );
}
```

### Working with Disks

Similarly, Disks API has changed. Use `Disks::new_with_refreshed_list()` instead:

```rust
use sysinfo::{Disks, DiskExt};

// Create Disks instance with refreshed data
let disks = Disks::new_with_refreshed_list();

// Iterate through disks
for disk in &disks {
    println!(
        "{}: {} bytes total, {} bytes available",
        disk.mount_point().display(),
        disk.total_space(),
        disk.available_space()
    );
}
```

### Process Information

The process API has several changes, particularly in how thread information is accessed:

```rust
use sysinfo::{System, SystemExt, ProcessExt};

let mut system = System::new_all();
system.refresh_all();

// Iterate over processes
for (pid, process) in system.processes() {
    // Get thread information - note the change from thread_count to thread_kind
    let thread_count = match process.thread_kind() {
        Some(_) => 1, // In 0.30, thread_kind returns an enum, not a count
        None => 0,
    };
    
    println!(
        "Process: {} (PID: {}), CPU: {}%, Memory: {} bytes, Threads: {}",
        process.name(),
        pid,
        process.cpu_usage(),
        process.memory(),
        thread_count
    );
}
```

## Common Patterns for Version 0.30

### Refreshing Network Data

```rust
// Create a fresh Networks instance with refreshed data
let networks = Networks::new_with_refreshed_list();

// Collect network statistics
let mut network_stats = Vec::new();
for (name, network) in &networks {
    network_stats.push((
        name.clone(),
        network.received(),
        network.transmitted()
    ));
}
```

### Collecting Disk Information

Since Disk objects don't implement Clone in 0.30, you need to handle them differently:

```rust
use sysinfo::{Disks, DiskExt};
use std::path::Path;

// Create fresh Disks instance with refreshed data
let disks = Disks::new_with_refreshed_list();

// Calculate storage usage for a specific path
let path = Path::new("/some/path");
let storage_usage = disks.iter()
    .filter(|disk| Path::new(disk.mount_point()).starts_with(path))
    .map(|disk| {
        let total = disk.total_space() as f64;
        let available = disk.available_space() as f64;
        if total > 0.0 {
            (total - available) / total
        } else {
            0.0
        }
    })
    .fold(0.0, |acc, x| acc + x);
```

### Working with Thread Information

The thread information API has changed significantly in 0.30:

```rust
use sysinfo::{System, SystemExt, ProcessExt};

let mut system = System::new_all();
system.refresh_all();

// Calculate total thread count
let thread_count: u32 = system.processes()
    .values()
    .map(|process| {
        match process.thread_kind() {
            Some(_) => 1, // In 0.30, thread_kind returns an enum, not a count
            None => 0,
        }
    })
    .sum();

println!("Total thread count: {}", thread_count);
```

## Asynchronous Usage with Tokio

When using sysinfo with async code, make sure to handle data refreshing properly:

```rust
use sysinfo::{System, Networks, Disks};
use tokio::sync::RwLock;
use std::sync::Arc;

// Shared system object
let system = Arc::new(RwLock::new(System::new_all()));

// Refresh networks asynchronously
async fn refresh_networks(system: Arc<RwLock<System>>) {
    // In version 0.30, it's better to create fresh instances
    // instead of using system.refresh_networks()
    let networks = Networks::new_with_refreshed_list();
    
    // Process network data
    for (name, network) in &networks {
        println!("{}: {}/{}", name, network.received(), network.transmitted());
    }
}
```

## Best Practices for sysinfo 0.30

1. **Refresh Before Reading**: Always refresh data before attempting to read it
2. **Use Fresh Instances**: For networks and disks, prefer creating fresh instances with `new_with_refreshed_list()`
3. **Handle Thread Information Carefully**: Remember that `thread_kind()` returns an enum, not a count
4. **Be Cautious with Disk References**: Since Disk doesn't implement Clone, work with references or copy the data you need
5. **Maintain Consistency**: Keep the same System instance around instead of recreating it multiple times
6. **Limit File Descriptor Usage**: If your program needs many file descriptors, use `sysinfo::set_open_files_limit(0)`

## Example: Complete Resource Monitoring

```rust
use sysinfo::{System, SystemExt, Networks, NetworkExt, Disks, DiskExt, ProcessExt};
use std::collections::HashMap;
use std::path::Path;

// Create resource metrics structure
struct ResourceMetrics {
    cpu_usage: f32,
    memory_usage: f64,
    storage_usage: f64,
    network_bandwidth: f64,
    thread_count: u32,
}

fn collect_system_metrics() -> ResourceMetrics {
    // Create new System instance
    let mut system = System::new_all();
    system.refresh_all();
    
    // Create fresh Networks instance
    let networks = Networks::new_with_refreshed_list();
    
    // Create fresh Disks instance
    let disks = Disks::new_with_refreshed_list();
    
    // Calculate CPU usage
    let cpu_usage = system.global_cpu_info().cpu_usage();
    
    // Calculate memory usage
    let memory_total = system.total_memory() as f64;
    let memory_used = system.used_memory() as f64;
    let memory_usage = if memory_total > 0.0 {
        memory_used / memory_total
    } else {
        0.0
    };
    
    // Calculate storage usage
    let storage_usage = disks.iter()
        .map(|disk| {
            let total = disk.total_space() as f64;
            let used = total - (disk.available_space() as f64);
            if total > 0.0 {
                used / total
            } else {
                0.0
            }
        })
        .fold(0.0, |acc, x| acc + x) / disks.iter().count() as f64;
    
    // Calculate network bandwidth
    let network_bandwidth = networks.iter()
        .map(|(_, network)| {
            let received = network.received() as f64;
            let transmitted = network.transmitted() as f64;
            received + transmitted
        })
        .fold(0.0, |acc, x| acc + x);
    
    // Calculate thread count
    let thread_count: u32 = system.processes()
        .values()
        .map(|process| {
            match process.thread_kind() {
                Some(_) => 1, // In 0.30, thread_kind returns an enum, not a count
                None => 0,
            }
        })
        .sum();
    
    ResourceMetrics {
        cpu_usage,
        memory_usage,
        storage_usage,
        network_bandwidth,
        thread_count,
    }
}
```

## Compatibility Notes

If you're migrating from a different version of sysinfo, be aware of these key differences in 0.30:

1. The `refresh_networks_list()` and `refresh_disks_list()` methods have been replaced with `refresh_networks()` and `refresh_disks()`
2. The `thread_count()` method has been replaced with `thread_kind()`
3. Some objects that previously implemented Clone no longer do
4. The System methods `networks()` and `disks()` have different semantics

## Further Resources

- [sysinfo GitHub Repository](https://github.com/GuillaumeGomez/sysinfo)
- [sysinfo Documentation](https://docs.rs/sysinfo)
- [sysinfo Crate](https://crates.io/crates/sysinfo)

## How We Use sysinfo in Our Project

In the Squirrel project, we use sysinfo 0.30 for monitoring system resources including network, disk usage, and process information. We've implemented several adapters and utilities to work with its API effectively:

1. `SystemInfoAdapter` - Provides access to system information with both synchronous and asynchronous interfaces
2. `NetworkMonitor` - Tracks network interface usage over time
3. `ResourceMetricsCollector` - Collects and aggregates system resource metrics

These components handle the specific requirements of sysinfo 0.30 including the proper refreshing of data and managing non-cloneable types. 
