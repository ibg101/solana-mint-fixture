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
solana-mint-fixture = { version = "0.1.0", features = ["banks"] }  # Enables BanksClient (Disabled by default)
solana-mint-fixture = { version = "0.1.0", features = ["rpc"] }    # Enables RpcClient   (Disabled by default)
solana-mint-fixture = { version = "0.1.0", features = ["full"] }   # Enable both         (Disabled by default)
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
let mint: Pubkey = fixture.create_and_initialize_mint_without_freeze(9, &blockhash).await?;
let ata: Pubkey = fixture.create_and_initialize_ata(&mint, &blockhash).await?;
fixture.mint_to_ata(&mint, &ata, 1_000_000_000, &blockhash).await?;
```