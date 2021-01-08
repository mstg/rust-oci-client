use crate::pod::initialize_pod_container_statuses;
use crate::pod::Pod;
use crate::provider::Provider;
use k8s_openapi::api::core::v1::Pod as KubePod;
use krator::state::SharedState;
use krator::ObjectState;
use krator::Operator;
use kube::Api;
use std::sync::Arc;

pub(crate) struct PodOperator<P: Provider> {
    provider: Arc<P>,
    client: kube::Client,
}

impl<P: Provider> PodOperator<P> {
    pub fn new(provider: Arc<P>, client: kube::Client) -> Self {
        PodOperator { provider, client }
    }
}

#[async_trait::async_trait]
impl<P: Provider> Operator for PodOperator<P> {
    type Manifest = crate::pod::Pod;
    type Status = crate::pod::Status;
    type ObjectState = P::PodState;
    type InitialState = P::InitialState;
    type DeletedState = P::TerminatedState;

    async fn initialize_object_state(&self, manifest: &Pod) -> anyhow::Result<P::PodState> {
        self.provider.initialize_pod_state(manifest).await
    }

    async fn shared_state(&self) -> SharedState<<P::PodState as ObjectState>::SharedState> {
        self.provider.provider_state()
    }

    async fn registration_hook(&self, manifest: SharedState<Self::Manifest>) -> anyhow::Result<()> {
        let (name, api) = {
            let initial_manifest = manifest.read().await.clone();
            let namespace = initial_manifest.namespace();
            let name = initial_manifest.name();
            let api: Api<KubePod> = Api::namespaced(self.client.clone(), namespace);
            (name.to_string(), api)
        };
        initialize_pod_container_statuses(name, manifest, &api).await
    }
}
