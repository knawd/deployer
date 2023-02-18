use env_logger::Env;
use log::{error, info};
use std::env;
use std::path::Path;
use std::{thread, time};

// keeping this incase additional libraries need copying
static VENDOR_BASE: &str = "vendor";
#[allow(clippy::while_immutable_condition)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut lib_files = vec![];

    // An output test so the build can be validated
    if env::args().count() >= 2 && env::args().nth(1) != Some("remove".to_string()) {
        println!("Manager 0.1 by Anton Whalley");
        std::process::exit(0);
    }
    for (key, value) in env::vars() {
        info!("{key}: {value}");
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // Forcing unwraps here as these values should be set and a failure will leave the node unusable
    let vendor = env::var("VENDOR").expect("VENDOR env should be set");
    let node_root = env::var("NODE_ROOT").expect("NODE_ROOT env should be set");
    let lib_location = format!(
        "{}{}",
        node_root,
        env::var("LIB_LOCATION").expect("LIB_LOCATION env should be set")
    );
    let config_location = format!(
        "{}{}",
        node_root,
        env::var("CONFIG_LOCATION").expect("CONFIG_LOCATION env should be set")
    );
    let oci_location = format!(
        "{}{}",
        node_root,
        env::var("OCI_LOCATION").expect("OCI_LOCATION env should be set")
    );

    let oci_type = env::var("OCI_TYPE").expect("OCI_TYPE env should be set");

    no_path_exit(&node_root);
    no_path_exit(&lib_location);
    no_path_exit(&config_location);
    no_path_exit(&oci_location);

    if oci_type == *"crun-wasmedge" {
        lib_files.push("libwasmedge.so.0");
    }
    if oci_type == *"crun-wasmtime" {
        lib_files.push("libwasmtime.so");
    }

    if oci_type == *"crun-wasm-nodejs" {
        lib_files.push("libnode.so");
    }
    let auto_restart = env::var("AUTO_RESTART")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        .parse()
        .unwrap_or(false);

    let is_micro_k8s: bool = env::var("IS_MICROK8S")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        .parse()
        .unwrap_or(false);
    let loopi: bool = env::var("LOOP")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        .parse()
        .unwrap_or(true);
    let patch_knative: bool = env::var("PATCH_KNATIVE")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        .parse()
        .unwrap_or(true);
    info!(
        "Starting manager with vendor: {} isMicroK8s: {} autorestart: {} node_root: {}",
        vendor, is_micro_k8s, auto_restart, node_root
    );
    // everything is as we expected so lets see if this is a remove
    if env::args().nth(1) == Some("remove".to_string()) {
        info!("Running the remove container");
        match vendor.as_str() {
            "rhel8" => {
                for file_name in &lib_files {
                    let deployed_file = format!("{lib_location}/{file_name}");
                    manager::delete_file(&deployed_file)?;
                }
                let crio_file = format!("{config_location}/crio.conf");
                manager::restore_crio_config(crio_file.as_str())?;
                let oci_file = format!("{oci_location}/crun");
                manager::delete_file(&oci_file)?;
                let oci_bak = format!("{config_location}/crio.conf.bak");
                manager::delete_file(&oci_bak)?;
                if auto_restart {
                    manager::restart_oci_runtime(node_root, is_micro_k8s, "crio".to_string())?;
                }
            }
            "ubuntu_20_04" | "ubuntu_18_04" => {
                for file_name in &lib_files {
                    let deployed_file = format!("{lib_location}/{file_name}");
                    manager::delete_file(&deployed_file)?;
                }
                let toml_file = format!("{config_location}/config.toml");
                manager::restore_containerd_config(toml_file.as_str())?;
                let oci_file = format!("{oci_location}/crun");
                manager::delete_file(&oci_file)?;
                let oci_bak = format!("{config_location}/config.toml.bak");
                manager::delete_file(&oci_bak)?;

                if auto_restart {
                    manager::restart_oci_runtime(
                        node_root,
                        is_micro_k8s,
                        "containerd".to_string(),
                    )?;
                }
            }
            _ => panic!(
                "unknown vendor {vendor} use either `rhel8` `ubuntu_20_04` or `ubuntu_18_04`"
            ),
        };
        if patch_knative {
            match manager::set_runtimeclassname("disabled").await {
                Ok(_v) => {}
                Err(e) => {
                    error!("Failed to patch knative: {e}\n Consider running :\n kubectl patch configmap/config-features -n knative-serving --type merge --patch '{{\"data\":{{\"kubernetes.podspec-runtimeclassname\":\"disabled\"}}}}'")
                }
            }
        }
        return Ok(());
    }

    manager::copy_to(VENDOR_BASE, oci_location.as_str(), &vendor, &oci_type)?;
    let full_oci_location = format!("{oci_location}/crun");
    let host_oci_location = full_oci_location.replace(&node_root, "");
    match vendor.as_str() {
        "rhel8" => {
            for file_name in &lib_files {
                manager::copy_to(VENDOR_BASE, lib_location.as_str(), &vendor, file_name)?;
            }
            let crio_file = format!("{config_location}/crio.conf");
            manager::update_crio_config(crio_file.as_str(), host_oci_location.as_str())?;
            if auto_restart {
                manager::restart_oci_runtime(node_root, is_micro_k8s, "crio".to_string())?;
            }
        }
        "ubuntu_20_04" | "ubuntu_18_04" => {
            for file_name in &lib_files {
                manager::copy_to(VENDOR_BASE, lib_location.as_str(), &vendor, file_name)?
            }

            let toml_file = if is_micro_k8s {
                format!("{config_location}/containerd.toml")
            } else {
                format!("{config_location}/config.toml")
            };
            manager::update_containerd_config(toml_file.as_str(), host_oci_location.as_str())?;
            if auto_restart {
                manager::restart_oci_runtime(node_root, is_micro_k8s, "containerd".to_string())?;
            }
        }
        _ => panic!("unknown vendor {vendor} use either `rhel8` `ubuntu_20_04` or `ubuntu_18_04`"),
    };

    if patch_knative {
        match manager::set_runtimeclassname("enabled").await {
            Ok(_v) => {}
            Err(e) => {
                error!("Failed to patch knative: {e}\n Consider running :\n kubectl patch configmap/config-features -n knative-serving --type merge --patch '{{\"data\":{{\"kubernetes.podspec-runtimeclassname\":\"enabled\"}}}}'")
            }
        }
    }

    let delay = time::Duration::from_millis(1000);
    while loopi {
        thread::sleep(delay);
    }
    Ok(())
}

fn no_path_exit(path: &str) {
    if !Path::new(&path).exists() {
        panic!("Exiting: {path} Does not exist");
    }
}
