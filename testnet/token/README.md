# Ekiden Testnet for token contract

This is a simple Ekiden testnet implemented using a single Kubernetes cluster. You can deploy it on a local Kubernetes installation by using [minikube](https://github.com/kubernetes/minikube) (see link for installation instructions).

Once you have your Kubernetes installation running and `kubectl` installed you can use the following commands:

To deploy:
```bash
$ make create
```

To destroy:
```bash
$ make destroy
```

Note that the destroy command may take some time to complete and may return a timeout. In this case, just run it again and wait until it completes successfully.

## Getting the Ekiden compute node IP and port

If you are using minikube, you can use the following command to get the correct IP and port you need to point your Ekiden client to:
```bash
$ minikube service --url ekiden-token-proxy
```

## Building the ekiden/core image

The testnet uses the `ekiden/core` Docker image, which contains prebuilt Ekiden binaries and contracts. In order to (re)build this Docker image, you can run the following command in the top-level Ekiden directory:
```bash
$ ./docker/deployment/build-images.sh
```

This will build `ekiden/core` locally and you can then push the image to your preferred registry.
