use crate::error::{NetworkError, Result};
use crate::soroban_rpc_client::{
    SendTransactionResponse, SimulateTransactionResponse, SorobanRpcClient,
};
use metrics::counter;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct SorobanService {
    rpc_client: Arc<SorobanRpcClient>,
}

impl SorobanService {
    pub fn new(rpc_client: Arc<SorobanRpcClient>) -> Self {
        Self { rpc_client }
    }

    /// Simulate a Soroban transaction
    pub async fn simulate_transaction(
        &self,
        transaction_xdr: &str,
    ) -> Result<SimulateTransactionResponse> {
        debug!("Simulating Soroban transaction");
        self.rpc_client
            .simulate_transaction(transaction_xdr)
            .await
            .map_err(|e| {
                counter!("soroban_rpc_errors_total", 1);
                e
            })
    }

    /// Submit a Soroban transaction
    pub async fn submit_transaction(
        &self,
        transaction_xdr: &str,
    ) -> Result<SendTransactionResponse> {
        info!("Submitting Soroban transaction");
        let response = self
            .rpc_client
            .send_transaction(transaction_xdr)
            .await
            .map_err(|e| {
                counter!("soroban_rpc_errors_total", 1);
                e
            })?;

        match response.status.as_str() {
            "ERROR" => {
                counter!("soroban_rpc_errors_total", 1);
                if let Some(xdr) = &response.error_result_xdr {
                    // Simple heuristic for error parsing without full XDR decoding in this example
                    if xdr.contains("AAAAAAAAAAAAAAAB") {
                        // Placeholder for expired
                        return Err(NetworkError::TransactionExpired);
                    }
                    if xdr.contains("AAAAAAAAAAAAAAAC") {
                        // Placeholder for insufficient fee
                        return Err(NetworkError::InsufficientFee);
                    }
                }
                Err(NetworkError::SorobanRpc(format!(
                    "Transaction failed: {}",
                    response.status
                )))
            }
            "PENDING" | "SUCCESS" => Ok(response),
            _ => {
                warn!("Unknown transaction status: {}", response.status);
                Ok(response)
            }
        }
    }

    /// Get the health of the Soroban RPC server
    pub async fn get_health(&self) -> Result<String> {
        let response = self.rpc_client.get_health().await?;
        Ok(response.status)
    }

    /// Get transaction status by hash
    pub async fn get_transaction_status(&self, hash: &str) -> Result<String> {
        let response = self.rpc_client.get_transaction(hash).await?;
        Ok(response.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SorobanConfig;
    use mockall::mock;
    use mockall::predicate::*;

    // In a real scenario we might use mockall for SorobanRpcClient if we define a trait
}
