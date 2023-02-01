# knawd-deployer

An chart to configure a kubernetes cluster to run WASM services.

## Caution
Installing with this helm chart will reboot the CRI runtime on the cluster.
If you would prefer to manually restart the CRI service run this chart with the `--set job.autorestart=false` option

## Ubuntu 18.04

```
cd charts/knawd-deployer
helm install knawd-deployer --create-namespace --namespace knawd .
```

## Ubuntu 20.04

```
cd charts/knawd-deployer
helm install knawd-deployer --create-namespace --namespace knawd -f ubuntu20-values.yaml .
```

## OpenShift

Red Hat Core OS based services have yet to be tested. We expect there to be some issues with copying the WASM libraries as they will need to be discovered by crun but it shouldn't be impossible.

```
cd charts/knawd-deployer
helm install knawd-deployer --create-namespace --namespace knawd -f rhel8-values.yaml .
```

## Knative

```
By default patching knative is enabled. If knative is not installed on the cluster this service will log an error but continue to run
If you wish to use this chart to obtain a crun enabled cluster but without knative running use `-set daemonset.patchKnative=false`.
```

## Values

These are the values particular to the deployer service.

image:

  registry: The registry where the image is stored used to specifiy a custom image (default: quay.io)

  repository: The repository in the registry where the image is located used to specifiy a custom image (default: knawd/deployer)

  tag: The tag in the repository where the image is located used to specifiy a custom image  (default: latest)
  pullPolicy: The pull policy for the image (default: Always)

daemonset:

  name: The name of the job (default: "knawd-deployer")

  vendor: The type of node the job will be deployed on (default: "ubuntu_18_04")

  libLocation: The location where the external WASM library will be copied to so that crun can find it (default: "/lib")

  logLevel: The log level (default: "info")

  ociLocation: The location on the host where the custom crun build will be placed (default: "/usr/local/sbin")

  configLocation: The location of the OCI configuration on the node (default: "/etc/containerd")

  ociType: The type of the OCI Runtime to deploy. Currently "crun-wasmedge" and "crun-wasmtime" are supported (default: "crun-wasmedge")

  nodeRoot: The location in the deployer where the node file system is mounted (default: "/mnt/node-root")

  isMicroK8s: Is this a microK8s installation (default: false)

  autoRestart: Should the deployer automatically restart the CRI service? Required for the config to be applied (default: true)

  patchKnative: Runs the patch to enable setting the runtime in a knative service definition.