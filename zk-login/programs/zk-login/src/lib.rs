mod account;
use account::*;
mod error;
use error::*;
mod verify_key;
use verify_key::VERIFYINGKEY;

use anchor_lang::prelude::*;
use groth16_solana::groth16::Groth16Verifier;

declare_id!("EgM2WEYJ6cDjkEmRvN6S5t2CgeJYaRLWK82qwwDBaSBy");

const MAX_HASH_LENGTH: usize = 100;
const DISCRIMINATOR: usize = 8;
const NR_PUBLIC_INPUTS: usize = 1;

#[program]
pub mod zk_login {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn user_sign_up(ctx: Context<SignUp>, hash: String) -> Result<()> {
        require!(hash.len() <= MAX_HASH_LENGTH, AuthenticationError::HashTooLong);
        let sing_up_data = &mut ctx.accounts.user_data;
        sing_up_data.hash = hash;
        msg!("user hash: {:?}", sing_up_data.hash);
        Ok(())
    }

    pub fn user_sign_in(ctx: Context<SignIn>, instruction_data: Vec<u8>) -> Result<()> {
        let mut public_inputs = [[0u8; 32]; NR_PUBLIC_INPUTS];
        public_inputs[0] = instruction_data[256..288].try_into().unwrap();
        msg!("public inputs: {:?}", &public_inputs);
        let proof_a = instruction_data[0..64].try_into().unwrap();
        msg!("pi_a: {:?}", proof_a);
        let proof_b = instruction_data[64..192].try_into().unwrap();
        msg!("pi_b: {:?}", proof_b);
        let proof_c = instruction_data[192..256].try_into().unwrap();
        msg!("pi_c: {:?}", proof_c);
        let mut verifier = Groth16Verifier::new(
            &proof_a,
            &proof_b,
            &proof_c,
            &public_inputs,
            &VERIFYINGKEY
        ).map_err(|_| ProgramError::Custom(0))?; // Use a custom error code
        let result = verifier.verify();
        match result {
            Ok(true) => msg!("Verification succeeded"),
            Ok(false) => msg!("Verification failed"),
            Err(e) => msg!("Verification error: {:?}", e),
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct SignUp<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"user_data", user.key().as_ref()],
        bump,
        space = DISCRIMINATOR + UserData::INIT_SPACE
    )]
    pub user_data: Account<'info, UserData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SignIn {

}
