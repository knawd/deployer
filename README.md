# deployer

Deploy a Web Assembly enabled container runtime into kubernetes to enable the use of WASM services in public, private and edge scenarios.

See https://knawd.dev for more information on how to build Web Assembly services using this system.


[![build status](https://github.com/knawd/deployer/workflows/CI/badge.svg)](https://github.com/knawd/deployer/actions)
[![OpenSSF Best Practices](https://bestpractices.coreinfrastructure.org/projects/6966/badge)](https://bestpractices.coreinfrastructure.org/projects/6966)
[![Artifact Hub](https://img.shields.io/endpoint?url=https://artifacthub.io/badge/repository/deployer)](https://artifacthub.io/packages/search?repo=deployer)


## Overview

This project deploys a custom build of the latest [crun release](https://github.com/containers/crun) with [WASMEdge](https://github.com/WasmEdge/WasmEdge)/[wasmtime](https://wasmtime.dev/)/[wasm-nodejs](https://github.com/mhdawson/crun/tree/node-wasm-experiment) support and provides a helm chart and executable to assist with the deployment.

## Install

Please see the instructions in [the helm chart README](https://github.com/knawd/deployer/blob/main/charts/knawd-deployer/README.md).

## Topology
![topology](https://raw.githubusercontent.com/knawd/documentation/af55ffa06a1d9c69dd827dca5991872dbde8dcc5/scenarios/images/topology.png)


The node configuration is preformed by a container deployed on each node by the daemonset. 

The container copies files to 3 locations on each node, restarts the container runtime service (crio or containerd) and applies a knative configuration:

1. The `/lib` or `/lib64` folder contains the shared objects `libwasmedge.so.0` or `libwasmtime.so` or `libnode.so`

1. The `/usr/local/sbin` folder to deploy the OCI executable e.g. `crun`

1. The additional runtime configuration is added to either the `crio.conf` or the containerd `config.toml`

1. The namespace role grants access to the host system while the cluster roles grants access to the config map resources.

### Secuirty Considerations

* The daemonset requires access to the host OS and uses the hostpid to restart the host runtimes
* A cluster role is used to update config maps
* It's strongly recommended that no other workloads are deployed into the same namespace

## Supported Versions

### Runtimes
|Release|WASMEdge|WASMtime|node-wasm|crun|
|---|---|---|---|---|
|v1.2.0|0.11.2|5.0.0|[experiment](https://github.com/mhdawson/crun/commit/23f346e3bc15ec7e6188b405df895aef5a5cbcdd)|[1.8](https://github.com/containers/crun/releases/tag/1.8)|

### Kubernetes Versions
|Ubuntu|OpenShift|microk8s|
|---|---|---|
|18.04, 20.04|4.10, 4.11|1.26.1|

N.B. Red Hat Core OS based instances have still to be tested and we expect some issues modifying the crio config and copying the WASM libs to the host.

## Contributions

Please read the [CONTRIBUTING.md](CONTRIBUTING.md) it has some important notes.
Pay specific attention to the **Coding style guidelines** and the **Developer Certificate of Origin**

## Code Of Conduct

We as members, contributors, and leaders pledge to make participation in our
community a harassment-free experience for everyone, regardless of age, body
size, visible or invisible disability, ethnicity, sex characteristics, gender
identity and expression, level of experience, education, socio-economic status,
nationality, personal appearance, race, religion, or sexual identity
and orientation.

We pledge to act and interact in ways that contribute to an open, welcoming,
diverse, inclusive, and healthy community.

[The full code of conduct is available here](./code-of-conduct.md)
