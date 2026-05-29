use std::path::Path;

use sysinfo::System;

use crate::adapters::types::DiscoveredProcess;

pub fn scan_processes(needles: &[&str]) -> Vec<DiscoveredProcess> {
    let mut system = System::new_all();
    system.refresh_all();

    system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let name = process.name().to_string();
            let command = process.cmd().join(" ");
            let haystack = format!("{} {}", name.to_lowercase(), command.to_lowercase());

            if !needles.iter().any(|needle| haystack.contains(&needle.to_lowercase())) {
                return None;
            }

            Some(DiscoveredProcess {
                pid: pid.as_u32(),
                name,
                command,
                cwd: process.cwd().map(Path::to_path_buf).map(|path| path.display().to_string()),
            })
        })
        .collect()
}
