use alloy::{
    eips::BlockId, primitives::B256, providers::ProviderBuilder, transports::mock::Asserter,
};
use risc0_zkvm::ProverOpts;
use test_log::test;
use zeth_host::BlockProcessor;

#[test(tokio::test)]
async fn prove_with_dev_mode() -> anyhow::Result<()> {
    // create a mock provider for the test input
    let s = std::fs::read_to_string(
        "testdata/0xe8c8bbefe0d6c4cbb426d1ab57e6f7cca9cda9405b237252b74525b9948b3e89.json",
    )?;
    let responses: Vec<&serde_json::value::RawValue> = serde_json::from_str(&s)?;

    let asserter = Asserter::new();
    for response in &responses {
        asserter.push_success(response);
    }
    let provider = ProviderBuilder::new().connect_mocked_client(asserter);

    // create the input
    let processor = BlockProcessor::new(provider).await?;
    let (input, block_hash) = processor.create_input(BlockId::latest()).await?;

    // prove in dev mode
    let (receipt, _) =
        processor.prove_with_opts(input, None, ProverOpts::default().with_dev_mode(true)).await?;

    let proven_hash = B256::try_from(receipt.journal.as_ref())?;
    assert_eq!(proven_hash, block_hash, "journal output mismatch");

    Ok(())
}
