# Solana Mint Fixture

**Solana Mint Fixture** is a lightweight utility library for creating and initializing SPL Token 2022 mints and ATAs in both Solana BanksClient based tests and real RpcClient environments.

## Features

- Create and initialize a new `Mint` account
- Derive and create an `Associated Token Account` (ATA)
- Mint tokens to the ATA
- Works with:
  - `BanksClient` (for Solana program tests)
  - `RpcClient`   (for integration/e2e testing)

## Usage

Add to your `Cargo.toml`:

```toml
# Enables only `banks` feature
solana-mint-fixture = { version = "0.1.0", default-features = false, features = ["banks"] }

# Enables only `rpc` feature
solana-mint-fixture = { version = "0.1.0", default-features = false, features = ["rpc"] }

# Enables `full` features (by default)
solana-mint-fixture = "0.1.0"
```

## Example
```rust
let fixture: MintFixture = MintFixture::new(
    MintFixtureClient::Banks(&banks_client),
    &payer_keypair,
    &payer_pubkey,
    &rent,
);

// make sure that the blockhash does not expire!
let decimals: u8 = 9;
let mint: Pubkey = fixture.create_and_initialize_mint_without_freeze(decimals, &blockhash).await?;
let ata: Pubkey = fixture.create_and_initialize_ata(&mint, &blockhash).await?;
fixture.mint_to_ata(&mint, &ata, 1_000_000_000 * 10u64.pow(decimals as u32), &blockhash).await?;
```