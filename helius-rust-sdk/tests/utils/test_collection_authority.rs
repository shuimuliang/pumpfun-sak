#[cfg(test)]
mod tests {
    use helius::utils::collection_authority::*;
    use mpl_token_metadata::ID;
    use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};

    #[test]
    fn test_get_collection_authority_record() {
        let collection_mint = Pubkey::new_unique();
        let collection_authority = Pubkey::new_unique();

        let result = get_collection_authority_record(&collection_mint, &collection_authority);

        let (expected_pubkey, _bump_seed) = Pubkey::find_program_address(
            &[
                b"metadata",
                ID.as_ref(),
                &collection_mint.to_bytes(),
                b"collection_authority",
                &collection_authority.to_bytes(),
            ],
            &ID,
        );

        assert_eq!(result, expected_pubkey);
    }

    #[test]
    fn test_get_collection_metadata_account() {
        let collection_mint = Pubkey::new_unique();

        let result = get_collection_metadata_account(&collection_mint);

        let (expected_pubkey, _bump_seed) =
            Pubkey::find_program_address(&[b"metadata", ID.as_ref(), &collection_mint.to_bytes()], &ID);

        assert_eq!(result, expected_pubkey);
    }

    #[test]
    fn test_get_delegate_collection_authority_instruction() {
        let collection_mint = Pubkey::new_unique();
        let new_collection_authority = Pubkey::new_unique();
        let update_authority_keypair = Keypair::new();
        let payer_pubkey = Pubkey::new_unique();

        let instruction = delegate_collection_authority_instruction(
            collection_mint,
            new_collection_authority,
            &update_authority_keypair,
            payer_pubkey,
        );

        assert_eq!(instruction.program_id, ID);

        let collection_metadata = get_collection_metadata_account(&collection_mint);
        let collection_authority_record = get_collection_authority_record(&collection_mint, &new_collection_authority);

        let expected_accounts = vec![
            AccountMeta::new(collection_authority_record, false),
            AccountMeta::new_readonly(new_collection_authority, false),
            AccountMeta::new(update_authority_keypair.pubkey(), true),
            AccountMeta::new(payer_pubkey, true),
            AccountMeta::new_readonly(collection_metadata, false),
            AccountMeta::new_readonly(collection_mint, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
        ];

        assert_eq!(instruction.accounts, expected_accounts);
    }

    #[test]
    fn test_get_revoke_collection_authority_instruction() {
        let collection_mint = Pubkey::new_unique();
        let collection_authority = Pubkey::new_unique();
        let revoke_authority_keypair = Keypair::new();

        let instruction =
            revoke_collection_authority_instruction(collection_mint, collection_authority, &revoke_authority_keypair);

        assert_eq!(instruction.program_id, ID);

        let expected_accounts = vec![
            AccountMeta::new(
                get_collection_authority_record(&collection_mint, &collection_authority),
                false,
            ),
            AccountMeta::new(collection_authority, false),
            AccountMeta::new(revoke_authority_keypair.pubkey(), true),
            AccountMeta::new_readonly(get_collection_metadata_account(&collection_mint), false),
            AccountMeta::new_readonly(collection_mint, false),
        ];

        assert_eq!(instruction.accounts, expected_accounts);
    }
}
