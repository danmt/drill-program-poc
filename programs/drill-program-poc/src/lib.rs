use {anchor_lang::prelude::*, anchor_spl::token::*};

declare_id!("7CUSJe6g6pbCjyyM5rQEBNPexqp8QTN1wQEdMLSkUFjx");

#[program]
pub mod drill_program_poc {
    use super::*;

    pub fn initialize_board(
        ctx: Context<InitializeBoard>,  
        board_id: u32
    ) -> Result<()> {
        ctx.accounts.board.initialize(
            ctx.accounts.authority.key(),
            board_id,
            ctx.accounts.accepted_mint.key(), 
            *ctx.bumps.get("board").unwrap(), 
            *ctx.bumps.get("board_vault").unwrap()
        );
        Ok(())
    }

    pub fn set_authority(
        ctx: Context<SetBoardAuthority>,  
        _board_id: u32,
    ) -> Result<()> {
        ctx.accounts.board.set_authority(ctx.accounts.authority.key());
        Ok(())
    }

    pub fn initialize_bounty(
        ctx: Context<InitializeBounty>,  
        board_id: u32, 
        bounty_id: u32
    ) -> Result<()> {
        ctx.accounts.bounty.initialize(
            board_id, 
            bounty_id, 
            *ctx.bumps.get("bounty").unwrap(), 
            *ctx.bumps.get("bounty_vault").unwrap()
        );
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        _board_id: u32, 
        _bounty_id: u32,
        amount: u64,
    ) -> Result<()> {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sponsor_vault.to_account_info(),
                    to: ctx.accounts.bounty_vault.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn close_bounty(
        ctx: Context<CloseBounty>,  
        _board_id: u32, 
        _bounty_id: u32,
        bounty_hunter: Option<String>,
    ) -> Result<()> {
        ctx.accounts.bounty.close(bounty_hunter);
        Ok(())
    }

    pub fn set_bounty_hunter(
        ctx: Context<SetBountyHunter>,  
        _board_id: u32, 
        _bounty_id: u32,
        bounty_hunter: String,
    ) -> Result<()> {
        ctx.accounts.bounty.set_bounty_hunter(bounty_hunter);
        Ok(())
    }

    pub fn send_bounty(
        ctx: Context<SendBounty>,
        board_id: u32, 
        _bounty_id: u32,
        _bounty_hunter: String,
    ) -> Result<()> {
        let seeds = [
            b"board".as_ref(),
            &board_id.to_le_bytes(),
            &[ctx.accounts.board.board_bump],
        ];
        let signer = &[&seeds[..]];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.bounty_vault.to_account_info(),
                    to: ctx.accounts.user_vault.to_account_info(),
                    authority: ctx.accounts.board.to_account_info(),
                },
                signer
            ),
            ctx.accounts.bounty_vault.amount,
        )?;
        close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                CloseAccount {
                    account: ctx.accounts.bounty_vault.to_account_info(),
                    destination: ctx.accounts.board_authority.to_account_info(),
                    authority: ctx.accounts.board.to_account_info(),
                },
                signer
            )
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(board_id: u32)]
pub struct InitializeBoard<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 200,
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump,
    )]
    pub board: Box<Account<'info, Board>>,
    pub accepted_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [
            b"board_vault".as_ref(), 
            board.key().as_ref()
        ],
        bump,
        token::mint = accepted_mint,
        token::authority = board,
    )]
    pub board_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(board_id: u32)]
pub struct SetBoardAuthority<'info> {
    #[account(
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump = board.board_bump,
        constraint = board.authority == authority.key(),
    )]
    pub board: Box<Account<'info, Board>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(
    board_id: u32, 
    bounty_id: u32,
)]
pub struct InitializeBounty<'info> {
    #[account(
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump = board.board_bump,
    )]
    pub board: Box<Account<'info, Board>>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 200,
        seeds = [
            b"bounty".as_ref(), 
            board.key().as_ref(), 
            &bounty_id.to_le_bytes(),
        ],
        bump,
    )]
    pub bounty: Box<Account<'info, Bounty>>,
    pub accepted_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [
            b"bounty_vault".as_ref(),
            bounty.key().as_ref()
        ],
        bump,
        token::mint = accepted_mint,
        token::authority = board,
    )]
    pub bounty_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    board_id: u32, 
    bounty_id: u32,
)]
pub struct Deposit<'info> {
    #[account(
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump = board.board_bump,
    )]
    pub board: Box<Account<'info, Board>>,
    #[account(
        seeds = [
            b"bounty".as_ref(), 
            board.key().as_ref(),
            &bounty_id.to_le_bytes(),
        ],
        bump = bounty.bounty_bump,
    )]
    pub bounty: Box<Account<'info, Bounty>>,
    #[account(
        mut,
        seeds = [
            b"bounty_vault".as_ref(), 
            bounty.key().as_ref()
        ],
        bump = bounty.bounty_vault_bump,
    )]
    pub bounty_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = sponsor_vault.mint == board.accepted_mint
    )]
    pub sponsor_vault: Box<Account<'info, TokenAccount>>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(
    board_id: u32, 
    bounty_id: u32,
    bounty_hunter: Option<String>,
)]
pub struct CloseBounty<'info> {
    #[account(
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump = board.board_bump,
        constraint = board.authority == authority.key(),
    )]
    pub board: Box<Account<'info, Board>>,
    #[account(
        mut,
        seeds = [
            b"bounty".as_ref(), 
            board.key().as_ref(), 
            &bounty_id.to_le_bytes(),
        ],
        bump = bounty.bounty_bump
    )]
    pub bounty: Box<Account<'info, Bounty>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(
    board_id: u32, 
    bounty_id: u32,
    bounty_hunter: String,
)]
pub struct SetBountyHunter<'info> {
    #[account(
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump = board.board_bump,
        constraint = board.authority == authority.key(),
    )]
    pub board: Box<Account<'info, Board>>,
    #[account(
        mut,
        seeds = [
            b"bounty".as_ref(), 
            board.key().as_ref(), 
            &bounty_id.to_le_bytes(),
        ],
        bump = bounty.bounty_bump
    )]
    pub bounty: Box<Account<'info, Bounty>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(
    board_id: u32, 
    bounty_id: u32,
    bounty_hunter: String,
)]
pub struct SendBounty<'info> {
    #[account(
        seeds = [
            b"board".as_ref(), 
            &board_id.to_le_bytes(),
        ],
        bump = board.board_bump,
    )]
    pub board: Box<Account<'info, Board>>,
    #[account(
        mut,
        close = board_authority,
        seeds = [
            b"bounty".as_ref(), 
            board.key().as_ref(),
            &bounty_id.to_le_bytes(),
        ],
        bump = bounty.bounty_bump,
        constraint = bounty.is_closed,
        constraint = bounty.bounty_hunter == Some(bounty_hunter)
    )]
    pub bounty: Box<Account<'info, Bounty>>,
    #[account(
        mut,
        seeds = [
            b"bounty_vault".as_ref(), 
            bounty.key().as_ref()
        ],
        bump = bounty.bounty_vault_bump,
    )]
    pub bounty_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = user_vault.mint == board.accepted_mint
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = board_authority.key() == board.authority,
    )]
    pub board_authority: SystemAccount<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


#[account]
pub struct Board {
    pub authority: Pubkey,
    pub board_id: u32,
    pub accepted_mint: Pubkey,
    pub board_bump: u8,
    pub board_vault_bump: u8,
}

impl Board {
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        board_id: u32,
        accepted_mint: Pubkey,
        board_bump: u8,
        board_vault_bump: u8,
    ) -> () {
        self.authority = authority;
        self.board_id = board_id;
        self.accepted_mint = accepted_mint;
        self.board_bump = board_bump;
        self.board_vault_bump = board_vault_bump;
    }

    pub fn set_authority(
        &mut self,
        authority: Pubkey,
    ) -> () {
        self.authority = authority;
    }
}

#[account]
pub struct Bounty {
    pub board_id: u32, 
    pub bounty_id: u32,
    pub bounty_bump: u8,
    pub bounty_vault_bump: u8,
    pub bounty_hunter: Option<String>,
    pub is_closed: bool,
}

impl Bounty {
    pub fn initialize(
        &mut self,
        board_id: u32,
        bounty_id: u32,
        bounty_bump: u8,
        bounty_vault_bump: u8,
    ) -> () {
        self.board_id = board_id;
        self.bounty_id = bounty_id;
        self.bounty_bump = bounty_bump;
        self.bounty_vault_bump = bounty_vault_bump;
        self.bounty_hunter = None;
        self.is_closed = false;
    }

    pub fn close(
        &mut self,
        bounty_hunter: Option<String>
    ) -> () {
        self.is_closed = true;
        self.bounty_hunter = bounty_hunter;
    }

    pub fn set_bounty_hunter(
        &mut self,
        bounty_hunter: String
    ) -> () {
        self.bounty_hunter = Some(bounty_hunter);
    }
}

