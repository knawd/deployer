# knawd-deployer

A chart to configure a kubernetes cluster to run WASM services.

## Caution
Installing with this helm chart will reboot the CRI runtime on the cluster.
If you would prefer to manually restart the CRI service run this chart with the `--set job.autorestart=false` option

## Ubuntu 18.04

```
cd charts/knawd-deployer
helm install knawd-deployer --create-namespace --namespace knawd --set target=ubuntu_18_04 .
```

## Ubuntu 20.04

```
cd charts/knawd-deployer
helm install knawd-deployer --create-namespace --namespace knawd --set target=ubuntu_20_04 .
```

## OpenShift

Red Hat Core OS based services have yet to be tested. We expect there to be some issues with copying the WASM libraries as they will need to be discovered by crun but it shouldn't be impossible.

```
cd charts/knawd-deployer
helm install knawd-deployer --create-namespace --namespace knawd .
```

## Knative

By default patching knative is enabled. If knative is not installed on the cluster this service will log an error but continue to run
If you wish to use this chart to obtain a crun enabled cluster but without knative running use `-set daemonset.patchKnative=false`.


## Values

These are the values particular to the deployer service.

**target**: The type of kubernetes cluster to be configured. Supported versions are `ubuntu_18_04`, `ubuntu_20_04`, `microk8s` `rhel8` (default: rhel8)

**tag**: The tag in the repository where the image is located used to specifiy a custom image  (default: latest)

**autoRestart**: Should the deployer automatically restart the CRI service? Required for the config to be applied (default: true)

**logLevel**: The log level. Supported options `info`, `error`, `warn`, `debug` (default: "info") 

**ociType**: The type of the OCI Runtime to deploy. Currently `crun-wasmedge`, `crun-wasmtime` and `crun-wasm-nodejs` are supported (default: "crun-wasmedge")

**patchKnative**: Runs the patch to enable setting the runtime in a knative service definition.
