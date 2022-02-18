use mpl_token_metadata::instruction::create_metadata_accounts_v2;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::Mint;

#[derive(serde::Deserialize)]
struct Env {
    
    admin_wallet: String,
    mint_pair: String,
    cluster: url::Url,
}

fn main() {
    
    let env = envy::from_env::<Env>().expect("env fail");
    let keypair = Keypair::from_base58_string(&env.admin_wallet);
    let mint = Keypair::from_base58_string(&env.mint_pair);
    let client = RpcClient::new(env.cluster.to_string());


    let mut ixs: Vec<Instruction> = vec![];

    let mint_account_pubkey = create_mint_account(&mut ixs, &keypair, &mint, &client);

    let assoc_account_pubkey = create_assoc_account(&mut ixs, &keypair, &mint_account_pubkey);

    create_metadata_account(&mut ixs, &keypair, &mint_account_pubkey);

    mint_nft(
        &mut ixs,
        &keypair,
        &mint_account_pubkey,
        &assoc_account_pubkey,
    );

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let transaction: Transaction = Transaction::new_signed_with_payer(
        &ixs,
        Some(&keypair.pubkey()),
        &[&mint, &keypair],
        recent_blockhash,
    );

    client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .unwrap();

    println!("Done");
}

fn create_assoc_account(
    ixs: &mut Vec<Instruction>,
    keypair: &Keypair,
    mint_account_pubkey: &Pubkey,
) -> Pubkey {
    let assoc = spl_associated_token_account::get_associated_token_address(
        &keypair.pubkey(),
        &mint_account_pubkey,
    );

    let ix = spl_associated_token_account::create_associated_token_account(
        &keypair.pubkey(),
        &keypair.pubkey(),
        &mint_account_pubkey,
    );

    ixs.push(ix);

    assoc
}

fn create_mint_account(
    ixs: &mut Vec<Instruction>,
    wallet_keypair: &Keypair,
    mint_account: &Keypair,
    client: &RpcClient,
) -> Pubkey {
    let mint_account_pubkey = mint_account.pubkey();
    let wallet_pubkey = wallet_keypair.pubkey();

    let minimum_balance_for_rent_exemption = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .unwrap();

    let create_account_instruction: Instruction = solana_sdk::system_instruction::create_account(
        &wallet_pubkey,
        &mint_account_pubkey,
        minimum_balance_for_rent_exemption,
        Mint::LEN as u64,
        &spl_token::ID,
    );
    let initialize_mint_instruction: Instruction = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint_account_pubkey,
        &wallet_pubkey,
        None,
        0,
    )
    .unwrap();

    ixs.push(create_account_instruction);
    ixs.push(initialize_mint_instruction);

    mint_account_pubkey
}

fn mint_nft(
    ixs: &mut Vec<Instruction>,
    wallet_keypair: &Keypair,
    mint_account_pubkey: &Pubkey,
    assoc_pubkey: &Pubkey,
) {
    let wallet_pubkey = wallet_keypair.pubkey();

    let mint_to_instruction: Instruction = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_account_pubkey,
        &assoc_pubkey,
        &wallet_pubkey,
        &[&wallet_pubkey],
        1,
    )
    .unwrap();

    ixs.push(mint_to_instruction);

}

fn create_metadata_account(
    ixs: &mut Vec<Instruction>,
    wallet_keypair: &Keypair,
    mint_account_pubkey: &Pubkey,
) {
    let wallet_pubkey = wallet_keypair.pubkey();

    let (metadata_addr, _) = mpl_token_metadata::pda::find_metadata_account(&mint_account_pubkey);

    // Test Metadata
    let name = String::from("Glonky");
    let symbol = String::from("GLK");
    let uri = String::from("https://arweave.net/WmuRqH3p5hBlX0quGEfyJXj08XQzJS_llTPEnJpDcLs");

    let new_metadata_instruction = create_metadata_accounts_v2(
        //program_id: Pubkey,
        mpl_token_metadata::ID,
        //metadata_account: Pubkey,
        metadata_addr,
        //mint: Pubkey,
        *mint_account_pubkey,
        //mint_authority: Pubkey,
        wallet_pubkey,
        //payer: Pubkey,
        wallet_pubkey,
        //update_authority: Pubkey,
        wallet_pubkey,
        //name: String,
        name,
        //symbol: String,
        symbol,
        //uri: String,
        uri,
        //creators: Option<Vec<Creator>>,
        None,
        //seller_fee_basis_points: u16,
        0,
        //update_authority_is_signer: bool,
        false,
        //is_mutable: bool
        false,
        //collection: Option<Collection>,
        None,
        //uses: Option<Uses>
        None,
    );

    ixs.push(new_metadata_instruction);
}