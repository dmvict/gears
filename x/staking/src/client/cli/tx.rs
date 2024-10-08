use crate::{
    CommissionRates, CreateValidator, DelegateMsg, Description, EditDescription, EditValidator,
    Message as StakingMessage, RedelegateMsg, UndelegateMsg,
};
use anyhow::Result;
use clap::{Args, Subcommand};
use gears::{
    tendermint::types::proto::crypto::PublicKey as TendermintPublicKey,
    types::{
        address::{AccAddress, ValAddress},
        base::coin::UnsignedCoin,
        decimal256::Decimal256,
        uint::Uint256,
    },
};

#[derive(Args, Debug, Clone)]
pub struct StakingTxCli {
    #[command(subcommand)]
    pub command: StakingCommands,
}

/// Create new validator initialized with a self-delegation to it
#[derive(Args, Debug, Clone)]
pub struct CreateValidatorCli {
    /// The validator's Protobuf JSON encoded public key
    pub pubkey: TendermintPublicKey,
    /// Amount of coins to bond
    pub amount: UnsignedCoin,
    /// The validator's name
    pub moniker: String,
    /// The optional identity signature (ex. UPort or Keybase)
    #[arg(long)]
    pub identity: String,
    /// The validator's (optional) website
    #[arg(long)]
    pub website: String,
    /// The validator's (optional) security contact email
    #[arg(long)]
    pub security_contact: String,
    /// The validator's (optional) details
    #[arg(long)]
    pub details: String,
    /// The initial commission rate percentage
    /* 0.1 */
    #[arg(long, default_value_t = Decimal256::from_atomics(1u64, 1).expect("default is valid"))]
    pub commission_rate: Decimal256,
    /// The maximum commission rate percentage
    /* 0.2 */
    #[arg(long, default_value_t = Decimal256::from_atomics(2u64, 1).expect("default is valid"))]
    pub commission_max_rate: Decimal256,
    /// The maximum commission change rate percentage (per day)
    /* 0.01 */
    #[arg(long, default_value_t = Decimal256::from_atomics(1u64, 2).expect("default is valid"))]
    pub commission_max_change_rate: Decimal256,
    /// The minimum self delegation required on the validator
    #[arg(long, default_value_t = Uint256::one())]
    pub min_self_delegation: Uint256,
}

impl CreateValidatorCli {
    pub fn try_into_cmd(self, from_address: AccAddress) -> anyhow::Result<CreateValidator> {
        let CreateValidatorCli {
            pubkey,
            amount,
            moniker,
            identity,
            website,
            security_contact,
            details,
            commission_rate,
            commission_max_rate,
            commission_max_change_rate,
            min_self_delegation,
        } = self;

        let delegator_address = from_address.clone();
        let validator_address = ValAddress::from(from_address);
        let description = Description {
            moniker: moniker.to_string(),
            identity: identity.to_string(),
            website: website.to_string(),
            security_contact: security_contact.to_string(),
            details: details.to_string(),
        };
        let commission = CommissionRates::new(
            commission_rate,
            commission_max_rate,
            commission_max_change_rate,
        )?;

        let msg = CreateValidator {
            description,
            commission,
            min_self_delegation,
            delegator_address,
            validator_address,
            pubkey: pubkey.clone(),
            value: amount.clone(),
        };

        Ok(msg)
    }
}

#[derive(Subcommand, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum StakingCommands {
    CreateValidator(CreateValidatorCli),
    /// Edit an existing validator account
    EditValidator {
        /// The validator's name
        moniker: Option<String>,
        /// The optional identity signature (ex. UPort or Keybase)
        #[arg(long)]
        identity: Option<String>,
        /// The validator's (optional) website
        #[arg(long)]
        website: Option<String>,
        /// The validator's (optional) security contact email
        #[arg(long)]
        security_contact: Option<String>,
        /// The validator's (optional) details
        #[arg(long)]
        details: Option<String>,
        /// The initial commission rate percentage
        #[arg(long)]
        commission_rate: Option<Decimal256>,
        /// The minimum self delegation required on the validator
        #[arg(long)]
        min_self_delegation: Option<Uint256>,
    },
    /// Delegate liquid tokens to a validator
    Delegate {
        /// The validator account address
        validator_address: ValAddress,
        /// Amount of coins to bond
        amount: UnsignedCoin,
    },
    /// Redelegate illiquid tokens from one validator to another
    Redelegate {
        /// The validator account address from which sends coins
        src_validator_address: ValAddress,
        /// The validator account address that receives coins
        dst_validator_address: ValAddress,
        /// Amount of coins to redelegate
        amount: UnsignedCoin,
    },
    /// Unbond shares from a validator
    Unbond {
        /// The validator account address
        validator_address: ValAddress,
        /// Amount of coins to unbond
        amount: UnsignedCoin,
    },
}

pub fn run_staking_tx_command(
    args: StakingTxCli,
    from_address: AccAddress,
) -> Result<StakingMessage> {
    match &args.command {
        StakingCommands::CreateValidator(msg) => {
            let msg = StakingMessage::CreateValidator(msg.clone().try_into_cmd(from_address)?);

            Ok(msg)

            // genOnly, _ := fs.GetBool(flags.FlagGenerateOnly)
            // if genOnly {
            //     ip, _ := fs.GetString(FlagIP)
            //     nodeID, _ := fs.GetString(FlagNodeID)
            //
            //     if nodeID != "" && ip != "" {
            //         txf = txf.WithMemo(fmt.Sprintf("%s@%s:26656", nodeID, ip))
            //     }
            // }
            //
            // return txf, msg, nil
        }
        StakingCommands::EditValidator {
            moniker,
            identity,
            website,
            security_contact,
            details,
            commission_rate,
            min_self_delegation,
        } => {
            let validator_address = ValAddress::from(from_address);
            let description = EditDescription {
                moniker: moniker.clone(),
                identity: identity.clone(),
                website: website.clone(),
                security_contact: security_contact.clone(),
                details: details.clone(),
            };
            let msg = StakingMessage::EditValidator(EditValidator::new(
                description,
                *commission_rate,
                *min_self_delegation,
                validator_address,
            ));

            Ok(msg)
        }
        StakingCommands::Delegate {
            validator_address,
            amount,
        } => Ok(StakingMessage::Delegate(DelegateMsg {
            delegator_address: from_address.clone(),
            validator_address: validator_address.clone(),
            amount: amount.clone(),
        })),
        StakingCommands::Redelegate {
            src_validator_address,
            dst_validator_address,
            amount,
        } => Ok(StakingMessage::Redelegate(RedelegateMsg {
            delegator_address: from_address.clone(),
            src_validator_address: src_validator_address.clone(),
            dst_validator_address: dst_validator_address.clone(),
            amount: amount.clone(),
        })),
        StakingCommands::Unbond {
            validator_address,
            amount,
        } => Ok(StakingMessage::Undelegate(UndelegateMsg {
            delegator_address: from_address.clone(),
            validator_address: validator_address.clone(),
            amount: amount.clone(),
        })),
    }
}
