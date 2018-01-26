create:
	# Prepare secrets.
	@echo "==> Preparing secrets"
	@kubectl create secret generic ias \
		--from-file=spid=../../keys/attestation/spid \
		--from-file=pkcs12=../../keys/attestation/client.pfx

	@echo "==> Deploying nodes"
	@kubectl create -f token.yaml

destroy:
	@echo "==> Destroying nodes"
	@kubectl delete --ignore-not-found=true --now=true --force=true -f token.yaml
	@kubectl delete --ignore-not-found=true --now=true --force=true pvc -l app=ekiden-token

	@echo "==> Destroying secrets"
	@kubectl delete --ignore-not-found=true --now=true --force=true secret ias

.PHONY: create destroy
