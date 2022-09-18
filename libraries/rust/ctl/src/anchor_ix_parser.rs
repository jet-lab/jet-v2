use std::{collections::HashMap, fmt::Debug, io::Read, sync::Arc};

use anchor_lang::{idl::IdlAccount, AnchorDeserialize};
use anchor_syn::{
    codegen::program::common::SIGHASH_GLOBAL_NAMESPACE,
    idl::{
        EnumFields, Idl, IdlAccountItem, IdlField, IdlInstruction, IdlType, IdlTypeDefinitionTy,
    },
};
use anyhow::{anyhow, bail, Context, Result};
use borsh::BorshDeserialize;
use jet_rpc::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub struct ParsedInstruction {
    pub program: Pubkey,
    pub name: String,
    pub accounts: Vec<ParsedAccountInput>,
    pub data: DataValue,
}
pub enum ParsedAccountInput {
    Account(String, Pubkey),
    Group(String, Vec<ParsedAccountInput>),
}

pub struct AnchorParser {
    rpc: Arc<dyn SolanaRpcClient>,
    idls: HashMap<Pubkey, Idl>,
}

impl AnchorParser {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self {
            rpc,
            idls: HashMap::new(),
        }
    }

    pub async fn try_parse_instruction(
        &mut self,
        instruction: &Instruction,
    ) -> Result<ParsedInstruction> {
        if instruction.data.len() < 8 {
            bail!("instruction is not in the standard anchor format");
        }

        let idl = self
            .idls
            .get(&instruction.program_id)
            .ok_or_else(|| anyhow!("did not load idl for {}", &instruction.program_id))?;
        let reader = IdlReader::new(&self.idls, idl);

        let ix_def = reader
            .find_instruction_by_discriminator(&instruction.data[..8])
            .ok_or_else(|| {
                anyhow!(
                    "unknown instruction for program {}: {}",
                    &instruction.program_id,
                    bs58::encode(&instruction.data[..8]).into_string(),
                )
            })?;

        let data =
            DataValue::Struct(reader.parse_data_struct(&mut &instruction.data[8..], &ix_def.args)?);

        let accounts = reader.parse_accounts(
            &mut instruction.accounts.iter().map(|meta| meta.pubkey),
            &ix_def.accounts,
        )?;

        Ok(ParsedInstruction {
            data,
            accounts,
            name: ix_def.name.clone(),
            program: instruction.program_id,
        })
    }

    pub async fn load_idl(&mut self, program_id: &Pubkey) -> Result<()> {
        if self.idls.contains_key(program_id) {
            return Ok(());
        }

        let idl_account = IdlAccount::address(program_id);
        let account_data = self
            .rpc
            .get_account_data(&idl_account)
            .await
            .with_context(|| anyhow!("getting idl for {program_id}"))?;

        let idl_account_obj: IdlAccount = AnchorDeserialize::deserialize(&mut &account_data[8..])?;
        let mut decoder = flate2::read::ZlibDecoder::new(&idl_account_obj.data[..]);
        let mut uncompressed = vec![];
        decoder.read_to_end(&mut uncompressed)?;

        let idl = serde_json::from_slice(&uncompressed).with_context(|| {
            anyhow!(
                "deserializing IDL account {idl_account} ({} bytes)",
                account_data.len()
            )
        })?;

        self.idls.insert(*program_id, idl);
        Ok(())
    }
}

/// Simplified representation of anchor/program data
pub enum DataValue {
    Bool(bool),
    IntegerSigned(i128),
    IntegerUnsigned(u128),
    FloatingPoint(f64),
    String(String),
    Blob(Vec<u8>),
    PublicKey(Pubkey),
    Optional(Option<Box<DataValue>>),
    Array(Vec<DataValue>),
    Struct(Vec<(String, DataValue)>),
    EnumTuple(String, Vec<DataValue>),
    EnumStruct(String, Vec<(String, DataValue)>),
}

impl Debug for DataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::IntegerSigned(value) => write!(f, "{value}"),
            Self::IntegerUnsigned(value) => write!(f, "{value}"),
            Self::FloatingPoint(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Blob(value) => write!(f, "{value:?}"),
            Self::PublicKey(value) => write!(f, "{value}"),
            Self::Optional(value) => write!(f, "{value:?}"),
            Self::Array(value) => write!(f, "{value:?}"),
            Self::Struct(fields) => {
                let mut fmt_struct = f.debug_struct("");

                for (name, value) in fields {
                    fmt_struct.field(name, value);
                }

                fmt_struct.finish()
            }
            Self::EnumTuple(name, fields) => {
                let mut fmt_tuple = f.debug_tuple(name);

                for value in fields {
                    fmt_tuple.field(value);
                }

                fmt_tuple.finish()
            }
            Self::EnumStruct(name, fields) => {
                let mut fmt_struct = f.debug_struct(name);

                for (name, value) in fields {
                    fmt_struct.field(name, value);
                }

                fmt_struct.finish()
            }
        }
    }
}

struct IdlReader<'a> {
    idl: &'a Idl,
    loaded: &'a HashMap<Pubkey, Idl>,
}

impl<'a> IdlReader<'a> {
    fn new(loaded: &'a HashMap<Pubkey, Idl>, idl: &'a Idl) -> IdlReader<'a> {
        IdlReader { idl, loaded }
    }

    fn find_instruction_by_discriminator(&self, discriminator: &[u8]) -> Option<&IdlInstruction> {
        use heck::ToSnakeCase;

        self.idl.instructions.iter().find(|ixdef| {
            let expected_sig = anchor_syn::codegen::program::common::sighash(
                SIGHASH_GLOBAL_NAMESPACE,
                &ixdef.name.to_snake_case(),
            );

            &expected_sig[..] == discriminator
        })
    }

    fn parse_accounts(
        &self,
        to_parse: &mut impl Iterator<Item = Pubkey>,
        idl_accounts: &[IdlAccountItem],
    ) -> Result<Vec<ParsedAccountInput>> {
        idl_accounts
            .iter()
            .map(|item| -> Result<_> {
                match item {
                    IdlAccountItem::IdlAccount(idl_account) => to_parse
                        .next()
                        .ok_or_else(|| anyhow!("too few accounts"))
                        .map(|pubkey| {
                            ParsedAccountInput::Account(idl_account.name.clone(), pubkey)
                        }),

                    IdlAccountItem::IdlAccounts(more_idl_accounts) => self
                        .parse_accounts(to_parse, &more_idl_accounts.accounts)
                        .map(|accounts| {
                            ParsedAccountInput::Group(more_idl_accounts.name.clone(), accounts)
                        }),
                }
            })
            .collect()
    }

    fn parse_data_struct(
        &self,
        data: &mut &[u8],
        fields: &[IdlField],
    ) -> Result<Vec<(String, DataValue)>> {
        fields
            .iter()
            .map(|field| self.parse_data_field(data, field))
            .collect()
    }

    fn parse_data_field(&self, data: &mut &[u8], field: &IdlField) -> Result<(String, DataValue)> {
        let value = self
            .parse_data_value(data, &field.ty)
            .with_context(|| format!("parsing field {}", &field.name))?;

        Ok((field.name.clone(), value))
    }

    fn parse_data_array(
        &self,
        data: &mut &[u8],
        ty: &IdlType,
        length: usize,
    ) -> Result<Vec<DataValue>> {
        (0..length)
            .map(|_| self.parse_data_value(data, ty))
            .collect::<Result<_>>()
    }

    fn parse_data_value(&self, data: &mut &[u8], ty: &IdlType) -> Result<DataValue> {
        Ok(match ty {
            IdlType::Bool => DataValue::Bool(BorshDeserialize::deserialize(data)?),
            IdlType::I8 => {
                DataValue::IntegerSigned(<i8 as BorshDeserialize>::deserialize(data)? as i128)
            }
            IdlType::I16 => {
                DataValue::IntegerSigned(<i16 as BorshDeserialize>::deserialize(data)? as i128)
            }
            IdlType::I32 => {
                DataValue::IntegerSigned(<i32 as BorshDeserialize>::deserialize(data)? as i128)
            }
            IdlType::I64 => {
                DataValue::IntegerSigned(<i64 as BorshDeserialize>::deserialize(data)? as i128)
            }
            IdlType::I128 => DataValue::IntegerSigned(BorshDeserialize::deserialize(data)?),
            IdlType::U8 => {
                DataValue::IntegerUnsigned(<u8 as BorshDeserialize>::deserialize(data)? as u128)
            }
            IdlType::U16 => {
                DataValue::IntegerUnsigned(<u16 as BorshDeserialize>::deserialize(data)? as u128)
            }
            IdlType::U32 => {
                DataValue::IntegerUnsigned(<u32 as BorshDeserialize>::deserialize(data)? as u128)
            }
            IdlType::U64 => {
                DataValue::IntegerUnsigned(<u64 as BorshDeserialize>::deserialize(data)? as u128)
            }
            IdlType::U128 => DataValue::IntegerUnsigned(BorshDeserialize::deserialize(data)?),
            IdlType::F32 => {
                DataValue::FloatingPoint(<u32 as BorshDeserialize>::deserialize(data)? as f64)
            }
            IdlType::F64 => DataValue::FloatingPoint(BorshDeserialize::deserialize(data)?),
            IdlType::Bytes => DataValue::Blob(BorshDeserialize::deserialize(data)?),
            IdlType::PublicKey => DataValue::PublicKey(BorshDeserialize::deserialize(data)?),
            IdlType::String => DataValue::String(BorshDeserialize::deserialize(data)?),
            IdlType::Option(inner_ty) => {
                DataValue::Optional(match Option::<()>::deserialize(data)? {
                    None => None,
                    Some(_) => Some(Box::new(self.parse_data_value(data, inner_ty)?)),
                })
            }
            IdlType::Array(inner_ty, length) => {
                DataValue::Array(self.parse_data_array(data, inner_ty, *length)?)
            }
            IdlType::Vec(inner_ty) => {
                let length = u32::deserialize(data)? as usize;
                DataValue::Array(self.parse_data_array(data, inner_ty, length)?)
            }
            IdlType::Defined(name) => match self.find_defined_ty(name)? {
                IdlTypeDefinitionTy::Struct { fields } => {
                    DataValue::Struct(self.parse_data_struct(data, fields)?)
                }

                IdlTypeDefinitionTy::Enum { variants } => {
                    let variant_index = u8::deserialize(data)? as usize;

                    if variant_index >= variants.len() {
                        bail!("no definition for variant with index {variant_index}");
                    }

                    let variant = &variants[variant_index];

                    match &variant.fields {
                        None => DataValue::EnumTuple(variant.name.clone(), vec![]),
                        Some(EnumFields::Tuple(types)) => DataValue::EnumTuple(
                            variant.name.clone(),
                            types
                                .iter()
                                .map(|t| self.parse_data_value(data, t))
                                .collect::<Result<_>>()?,
                        ),
                        Some(EnumFields::Named(fields)) => DataValue::EnumStruct(
                            variant.name.clone(),
                            self.parse_data_struct(data, fields)?,
                        ),
                    }
                }
            },
        })
    }

    fn find_defined_ty(&self, name: &str) -> Result<&IdlTypeDefinitionTy> {
        if let Some(ty) = find_type_in_idl(self.idl, name) {
            return Ok(ty);
        }

        for (_, idl) in self.loaded.iter() {
            if let Some(ty) = find_type_in_idl(idl, name) {
                return Ok(ty);
            }
        }
        bail!("no definition found in IDL for type {name}");
    }
}

impl Debug for ParsedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&self.name)
            .field("accounts", &DisplayParsedAccounts(&self.accounts))
            .field("data", &self.data)
            .finish()
    }
}

struct DisplayParsedAccounts<'a>(&'a Vec<ParsedAccountInput>);

impl<'a> Debug for DisplayParsedAccounts<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut struct_fmt = f.debug_struct("");

        for parsed in self.0 {
            match parsed {
                ParsedAccountInput::Account(name, key) => struct_fmt.field(name, key),
                ParsedAccountInput::Group(name, keys) => struct_fmt.field(name, &Self(keys)),
            };
        }

        struct_fmt.finish()
    }
}

fn find_type_in_idl<'a>(idl: &'a Idl, name: &str) -> Option<&'a IdlTypeDefinitionTy> {
    idl.types
        .iter()
        .find(|tydef| tydef.name == name)
        .map(|tydef| &tydef.ty)
}
