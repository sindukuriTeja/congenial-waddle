use anchor_lang::prelude::*;

// This is the program's on-chain ID (public key).
// It will be generated when the program is deployed.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod pestivid_program {
    use super::*;

    // Instruction to create a user profile account.
    // This is called once when a user signs up.
    pub fn create_user_profile(
        ctx: Context<CreateUserProfile>,
        role: String,
        name: String,
        email: String,
        phone: String,
        member_since: String,
    ) -> Result<()> {
        let user_profile = &mut ctx.accounts.user_profile;
        user_profile.authority = ctx.accounts.user.key();
        user_profile.role = role;
        user_profile.name = name;
        user_profile.email = email;
        user_profile.phone = phone;
        user_profile.member_since = member_since;
        user_profile.video_count = 0;
        Ok(())
    }

    // Instruction to create a video metadata account.
    // This is called by a farmer after uploading a video to IPFS.
    pub fn create_video_metadata(
        ctx: Context<CreateVideoMetadata>,
        ipfs_cid: String,
        video_file_hash: String,
        crop: String,
        pesticide: String,
        location: String,
        pesticide_company: String,
        purpose: String,
    ) -> Result<()> {
        let video = &mut ctx.accounts.video;
        video.authority = ctx.accounts.user.key();
        video.ipfs_cid = ipfs_cid;
        video.video_file_hash = video_file_hash;
        video.crop = crop;
        video.pesticide = pesticide;
        video.location = location;
        video.pesticide_company = pesticide_company;
        video.purpose = purpose;
        video.upload_timestamp = Clock::get()?.unix_timestamp;
        video.is_listed = false;

        // Increment the user's video count
        let user_profile = &mut ctx.accounts.user_profile;
        user_profile.video_count = user_profile.video_count.checked_add(1).unwrap();

        Ok(())
    }

    // Instruction to create a marketplace listing.
    // This links a video to a price.
    pub fn create_listing(
        ctx: Context<CreateListing>,
        min_price: u64, // Prices in lamports (1 SOL = 1,000,000,000 lamports)
        max_price: u64,
    ) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        listing.authority = ctx.accounts.user.key();
        listing.video = ctx.accounts.video.key();
        listing.min_price = min_price;
        listing.max_price = max_price;
        listing.is_sold = false;

        // Mark the video as listed to prevent duplicate listings
        ctx.accounts.video.is_listed = true;

        Ok(())
    }

     // Instruction for a buyer to purchase a listing.
    pub fn purchase_listing(ctx: Context<PurchaseListing>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        let buyer = &ctx.accounts.buyer;
        let seller = &ctx.accounts.seller;

        // Ensure the listing is not already sold
        if listing.is_sold {
            // This should ideally return an error
            return err!(Errors::ListingAlreadySold);
        }

        // Transfer SOL from buyer to seller (simplified)
        // A real implementation would require the buyer to specify the offer price
        // and handle the lamport transfer correctly.
        let price_to_pay = listing.min_price; // Simplified: buyer pays min price

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &buyer.key(),
            &seller.key(),
            price_to_pay,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                buyer.to_account_info(),
                seller.to_account_info(),
            ],
        )?;

        // Mark the listing as sold
        listing.is_sold = true;

        Ok(())
    }
}

// --- Account Contexts ---
// These structs define the accounts required by each instruction.

#[derive(Accounts)]
pub struct CreateUserProfile<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 20) + (4 + 20) + 8, // Discriminator + Authority + Role + Name + Email + Phone + MemberSince + VideoCount
        seeds = [b"user_profile", user.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateVideoMetadata<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + (4 + 64) + (4 + 64) + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 20) + 8 + 1, // Space for all fields
        seeds = [b"video", user.key().as_ref(), &[user_profile.video_count]],
        bump
    )]
    pub video: Account<'info, VideoMetadata>,
    #[account(
        mut,
        seeds = [b"user_profile", user.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_profile: Account<'info, UserProfile>,
    #[account(mut, address = user_profile.authority)]
    pub user: Signer<'info>,
    /// CHECK: We are just checking that this is the same authority as the user profile
    pub authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateListing<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32 + 8 + 8 + 1, // Discriminator + Authority + Video Pubkey + Min Price + Max Price + Is Sold
        seeds = [b"listing", video.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    #[account(mut, has_one = authority)]
    pub video: Account<'info, VideoMetadata>,
    #[account(mut, address = video.authority)]
    pub user: Signer<'info>,
    /// CHECK: We are just checking that this is the same authority as the video
    pub authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchaseListing<'info> {
    #[account(mut, has_one = authority)]
    pub listing: Account<'info, Listing>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: This is the seller's account, we will transfer lamports to it.
    #[account(mut, address = listing.authority)]
    pub seller: AccountInfo<'info>,
    /// CHECK: We are just checking that this is the same authority as the listing
    pub authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


// --- On-Chain Data Structures (Accounts) ---

#[account]
pub struct UserProfile {
    pub authority: Pubkey,
    pub role: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub member_since: String,
    pub video_count: u64,
}

#[account]
pub struct VideoMetadata {
    pub authority: Pubkey,
    pub ipfs_cid: String,
    pub video_file_hash: String,
    pub crop: String,
    pub pesticide: String,
    pub location: String,
    pub pesticide_company: String,
    pub purpose: String,
    pub upload_timestamp: i64,
    pub is_listed: bool,
}

#[account]
pub struct Listing {
    pub authority: Pubkey,
    pub video: Pubkey,
    pub min_price: u64,
    pub max_price: u64,
    pub is_sold: bool,
}

// --- Errors ---
#[error_code]
pub enum Errors {
    #[msg("This listing has already been sold.")]
    ListingAlreadySold,
}
