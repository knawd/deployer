# deployer

Deploy a WASM enabled crun into kubernetes to enable running WASM libraries in container infrastructure.

See https://knawd.dev for more information.


[![build status](https://github.com/knawd/deployer/workflows/CI/badge.svg)](https://github.com/knawd/deployer/actions)
[![Docker Repository on Quay](https://quay.io/repository/knawd/deployer/status "Docker Repository on Quay")](https://quay.io/repository/knawd/deployer)
[![OpenSSF Best Practices](https://bestpractices.coreinfrastructure.org/projects/6966/badge)](https://bestpractices.coreinfrastructure.org/projects/6966)

## Overview

This project creates a build of the latest [WASMEdge](https://github.com/WasmEdge/WasmEdge) enabled [crun release](https://github.com/containers/crun).

This build is then bundled into a vendored container along with an executable to help with the deployment.

Finally there is a helm chart to deploy it into managed kubernetes services.

The deployment is performed copying files to 3 locations on each node:

1. The `/lib` folder to copy the shared objects `libwasmedge.so.0`

2. The `/usr/local/sbin` folder to deploy the `crun` executable

3. The additional runtime is added to either the `crio.conf` or the containerd `config.toml`

4. Uninstall performs a rollback of the CRI configuration.

## Supported Versions

|Release|WASMEdge|crun|Ubuntu|OpenShift|
|---|---|---|---|---|
|v1.0.0-alpha|0.11.2|[main](https://github.com/containers/crun/commit/26fe1383a05279935e67ee31e7ff10c43e7d87ea)|18.04, 20.04|4.10, 4.11|

N.B. Red Hat Core OS based instances have still to be tested and we expect some issues modifying the crio config and copying the WASM libs to the host.

## Install

If you are just want to get a test cluster up and running then you could start with [the helm chart README.md](https://github.com/knawd/deployer/blob/main/charts/knawd-deployer/README.md).

For a production install please refer to the [operator repo](https://github.com/knawd/operator)

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

)

