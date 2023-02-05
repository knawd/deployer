use k8s_openapi::api::core::v1::ConfigMap;
use log::{debug, info};
use serde_json::json;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use toml_edit::{value, Array, Document, Item, Table};

use kube::{
    api::{Api, Patch, PatchParams},
    Client,
};

pub fn copy_to(
    vendor_base: &str,
    destination_base: &str,
    vendor: &str,
    file_name: &str,
) -> Result<(), std::io::Error> {
    let location = format!("{vendor_base}/{vendor}/{file_name}");

    // Really software development is just about how neatly you can throw things away.
    // Here we are selecting which type of crun to deploy
    let destination: String = match file_name {
        "crun-wasmedge" | "crun-wasmtime" => format!("{destination_base}/crun"),
        _ => format!("{destination_base}/{file_name}"),
    };

    info!("Copying from {} to {}", location, destination);
    fs::copy(location, destination)?;
    Ok(())
}

pub fn update_containerd_config(
    path: &str,
    full_oci_location: &str,
) -> Result<toml_edit::Document, std::io::Error> {
    let conf = generate_containerd_config(path, full_oci_location)?;
    let value: toml_edit::easy::Value =
        toml_edit::easy::from_str(conf.to_string().as_str()).unwrap();
    let result = toml_edit::easy::to_string_pretty(&value).unwrap();

    let destination = path.replace(".toml", ".toml.bak");
    info!("Copying from {} to {}", path, destination);
    fs::copy(path, destination)?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    f.write_all(result.as_bytes())?;
    f.flush()?;
    Ok(conf)
}

pub fn update_crio_config(
    path: &str,
    full_oci_location: &str,
) -> Result<toml_edit::Document, std::io::Error> {
    info!("Generating crio config");
    let conf = generate_crio_config(path, full_oci_location)?;

    let value: toml_edit::easy::Value =
        toml_edit::easy::from_str(conf.to_string().as_str()).unwrap();
    let result = toml_edit::easy::to_string_pretty(&value).unwrap();
    info!("Starting replace..");
    let destination = path.replace(".conf", ".conf.bak");
    info!("Copying from {} to {}", path, destination);
    fs::copy(path, destination)?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    f.write_all(result.as_bytes())?;
    f.flush()?;
    Ok(conf)
}

pub fn generate_crio_config(
    path: &str,
    full_oci_location: &str,
) -> Result<toml_edit::Document, std::io::Error> {
    info!("Reading location: {}", path);
    let content = std::fs::read_to_string(path)?;

    let mut conf = content.parse::<Document>().expect("invalid doc");
    // TODO: Look at workloads table annotations?
    // let mut poda = Array::default();
    // poda.push("*.wasm.*");
    // poda.push("module.wasm.image/*");
    // poda.push("*.module.wasm.image");
    // poda.push("module.wasm.image/variant.*");

    let mut table = Table::default();
    table["runtime_type"] = value("oci");
    table["runtime_path"] = value(full_oci_location);
    table["runtime_root"] = value("/run/crun");
    conf["crio"]["runtime"]["runtimes"]["crun"] = Item::Table(table);
    Ok(conf)
}

pub fn generate_containerd_config(
    path: &str,
    full_oci_location: &str,
) -> Result<toml_edit::Document, std::io::Error> {
    let content = std::fs::read_to_string(path)?;

    let mut conf = content
        .parse::<Document>()
        .expect("Failed to parse containerd config");

    let mut poda = Array::default();
    poda.push("*.wasm.*");
    poda.push("module.wasm.image/*");
    poda.push("*.module.wasm.image");
    poda.push("module.wasm.image/variant.*");

    let mut table = Table::default();
    table["runtime_type"] = value("io.containerd.runc.v2");
    table["privileged_without_host_devices"] = value(false);
    table["pod_annotations"] = value(poda);

    let mut opt_table = Table::default();
    opt_table["BinaryName"] = value(full_oci_location);
    conf["plugins"]["io.containerd.grpc.v1.cri"]["containerd"]["runtimes"]["crun"] =
        Item::Table(table);
    conf["plugins"]["io.containerd.grpc.v1.cri"]["containerd"]["runtimes"]["crun"]["options"] =
        Item::Table(opt_table);
    Ok(conf)
}

pub fn restore_containerd_config(path: &str) -> Result<(), std::io::Error> {
    let from = path.replace(".toml", ".toml.bak");
    info!("Copying from {} to {}", from, path);
    fs::copy(from, path)?;
    Ok(())
}

pub fn restore_crio_config(path: &str) -> Result<(), std::io::Error> {
    let from = path.replace(".conf", ".conf.bak");
    info!("Copying from {} to {}", from, path);
    fs::copy(from, path)?;
    Ok(())
}

pub fn delete_file(path: &str) -> Result<(), std::io::Error> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn restart_oci_runtime(
    node_root: String,
    is_micro_k8s: bool,
    mut oci_runtime: String,
) -> Result<(), std::io::Error> {
    let mount_path = format!("-m{node_root}/proc/1/ns/mnt");

    if is_micro_k8s {
        oci_runtime = "snap.microk8s.daemon-containerd".to_string();
    }

    let args = vec![
        mount_path.as_str(),
        "--",
        "systemctl",
        "restart",
        oci_runtime.as_str(),
    ];
    let path = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/kubernetes/bin";
    info!("Restarting oci: {args:#?} using path {path}");

    let result = run_command_text(args, path);
    info!("{:?}", result);
    Ok(())
}

pub async fn set_runtimeclassname(val: &str) -> anyhow::Result<()> {
    update_knative_configmap("kubernetes.podspec-runtimeclassname", val).await
}
pub async fn update_knative_configmap(key: &str, val: &str) -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let configmaps: Api<ConfigMap> = Api::namespaced(client, "knative-serving");

    info!("Setting knative kubernetes.podspec-runtimeclassname in knative-serving");
    let patch = json!({"data":{key: val}});
    let patchparams = PatchParams::default();
    let p_patched = configmaps
        .patch("config-features", &patchparams, &Patch::Merge(&patch))
        .await?;
    let patch_response = p_patched.data.unwrap();
    let isenabled = patch_response
        .get("kubernetes.podspec-runtimeclassname")
        .unwrap();
    info!("knative kubernetes.podspec-runtimeclassname: {isenabled:?}");
    Ok(())
}

fn run_command_text(args: Vec<&str>, bin_path: &str) -> Result<String, String> {
    debug!("running {:?} {:?}", args, bin_path);
    // nsenter -m/mnt/node-root/proc/1/ns/mnt -- /usr/bin/pkill containerd
    let cmd = match Command::new("nsenter")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(&args)
        .spawn()
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("failed to execute nsenter {args:?} {e}"));
        }
    };
    let waiter = match cmd.wait_with_output() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("failed to execute nsenter {args:?} {e}"));
        }
    };

    let mut err_str = String::new();
    match waiter.stderr.as_slice().read_to_string(&mut err_str) {
        Err(e) => {
            return Err(format!(
                "stderr read error - failed to execute nsenter {args:?} {e}"
            ));
        }
        Ok(_) => {
            if !err_str.is_empty() {
                return Err(format!(
                    "stderr not empty - failed to execute nsenter {args:?} {err_str}"
                ));
            }
        }
    }

    let mut ok_str = String::new();
    match waiter.stdout.as_slice().read_to_string(&mut ok_str) {
        Err(e) => {
            return Err(format!(
                "stdout error - failed to execute nsenter {args:?} {e}"
            ));
        }
        Ok(_) => Ok(ok_str),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_crio_config_test() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/in_crio.conf");
        let backup_path = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/in_crio.conf.bak");
        let new_compare = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/out_crio.conf");
        let old_file_contents =
            fs::read_to_string(path).expect("Should have been able to read the file");

        update_crio_config(path, "/ocr_path/crun").unwrap();

        let new_file_contents =
            fs::read_to_string(path).expect("Should have been able to read the new_file_contents");
        let new_compare_contents = fs::read_to_string(new_compare)
            .expect("Should have been able to read the new_compare_contents");
        let backup_file_contents = fs::read_to_string(backup_path)
            .expect("Should have been able to read the backup_file_contents");

        // Test the new file is as expected
        assert_eq!(
            new_file_contents, new_compare_contents,
            "Test the new file is as expected"
        );

        // Test the backup is part of the orginal file
        assert_eq!(
            old_file_contents, backup_file_contents,
            "Test the backup is part of the orginal file"
        );

        restore_crio_config(path).unwrap();
        let restored_file_contents = fs::read_to_string(path)
            .expect("Should have been able to read the restored_file_contents");
        assert_eq!(
            old_file_contents, restored_file_contents,
            "Test the restoration"
        );

        fs::remove_file(backup_path).expect("Failed to remove tmp backup file");
    }

    #[test]
    fn generate_crio_config_test() {
        let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/in_crio.conf");
        let conf = generate_crio_config(test_file, "/ocr_path/crun").unwrap();
        assert_eq!(
            conf["crio"]["runtime"]["runtimes"]["crun"]["runtime_type"].as_str(),
            Some("oci")
        );
        assert_eq!(
            conf["crio"]["runtime"]["runtimes"]["crun"]["runtime_path"].as_str(),
            Some("/ocr_path/crun")
        );
        assert_eq!(
            conf["crio"]["runtime"]["runtimes"]["crun"]["runtime_root"].as_str(),
            Some("/run/crun")
        );
    }
    #[test]
    fn update_containerd_config_test() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/in_config.toml");
        let backup_path = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/in_config.toml.bak");
        let new_compare = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/out_config.toml");
        let old_file_contents =
            fs::read_to_string(path).expect("Should have been able to read the file");

        update_containerd_config(path, "/ocr_path/crun").unwrap();

        let new_file_contents =
            fs::read_to_string(path).expect("Should have been able to read the new_file_contents");
        let new_compare_contents = fs::read_to_string(new_compare)
            .expect("Should have been able to read the new_compare_contents");
        let backup_file_contents = fs::read_to_string(backup_path)
            .expect("Should have been able to read the backup_file_contents");

        // Test the new file is as expected
        assert_eq!(
            new_file_contents, new_compare_contents,
            "Test the new file is as expected"
        );

        // Test the backup is part of the orginal file
        assert_eq!(
            old_file_contents, backup_file_contents,
            "Test the backup is part of the orginal file"
        );

        restore_containerd_config(path).unwrap();
        let restored_file_contents = fs::read_to_string(path)
            .expect("Should have been able to read the restored_file_contents");
        assert_eq!(
            old_file_contents, restored_file_contents,
            "Test the restoration"
        );

        fs::remove_file(backup_path).expect("Failed to remove tmp backup file");
    }
    #[test]
    fn generate_containerd_config_test() {
        let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/mocks/in_config.toml");
        let conf = generate_containerd_config(test_file, "/ocr_path/crun").unwrap();
        assert_eq!(
            conf["plugins"]["io.containerd.grpc.v1.cri"]["containerd"]["runtimes"]["crun"]
                ["runtime_type"]
                .as_str(),
            Some("io.containerd.runc.v2")
        );
        assert_eq!(
            conf["plugins"]["io.containerd.grpc.v1.cri"]["containerd"]["runtimes"]["crun"]
                ["privileged_without_host_devices"]
                .as_bool(),
            Some(false)
        );

        let opt3 = conf["plugins"]["io.containerd.grpc.v1.cri"]["containerd"]["runtimes"]["crun"]
            ["pod_annotations"]
            .as_array()
            .unwrap()
            .get(3);

        assert_eq!(
            opt3.unwrap().as_str().unwrap(),
            "module.wasm.image/variant.*"
        );

        assert_eq!(
            conf["plugins"]["io.containerd.grpc.v1.cri"]["containerd"]["runtimes"]["crun"]
                ["options"]["BinaryName"]
                .as_str(),
            Some("/ocr_path/crun")
        );
    }
}
