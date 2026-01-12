use anchor_lang::prelude::*;

#[account]
pub struct VaultAuthority {
    pub authorized_programs: Vec<Pubkey>,
    pub admin: Pubkey,
    pub bump: u8,
}

impl VaultAuthority {
    pub const MAX_AUTHORIZED: usize = 10;
    
    pub const LEN: usize = 8 + 4 + (32 * Self::MAX_AUTHORIZED) + 32 + 1;

    pub fn is_authorized(&self, program: &Pubkey) -> bool {
        self.authorized_programs.contains(program)
    }

    pub fn add_program(&mut self, program: Pubkey) -> Result<()> {
        require!(
            !self.is_authorized(&program),
            crate::errors::VaultError::ProgramAlreadyAuthorized
        );
        
        require!(
            self.authorized_programs.len() < Self::MAX_AUTHORIZED,
            crate::errors::VaultError::MaxAuthorizedProgramsReached
        );
        
        self.authorized_programs.push(program);
        Ok(())
    }

    pub fn remove_program(&mut self, program: &Pubkey) -> Result<()> {
        let pos = self.authorized_programs
            .iter()
            .position(|p| p == program)
            .ok_or(error!(crate::errors::VaultError::ProgramNotAuthorized))?;
        
        self.authorized_programs.remove(pos);
        Ok(())
    }
}

