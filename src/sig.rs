use anyhow::{Context, Result};
use bytes::{Bytes, BytesMut};
use hmac::digest::block_buffer::Eager;
use hmac::digest::consts::U256;
use hmac::digest::core_api::{BlockSizeUser, BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore};
use hmac::digest::typenum::{IsLess, Le, NonZero};
use hmac::digest::{Digest, HashMarker};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Bytes>;
    fn verify(&self, data: &[u8]) -> Result<Bytes>;
}

pub struct HmacSigner<D>
where
    D: CoreProxy,
    D::Core: HashMarker
    + UpdateCore
    + FixedOutputCore
    + BufferKindUser<BufferKind=Eager>
    + Default
    + Clone,
    <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
    Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero,
{
    key: Vec<u8>,
    _marker: std::marker::PhantomData<D>,
}

impl <D> HmacSigner<D>
where
    D: CoreProxy,
    D::Core: HashMarker
    + UpdateCore
    + FixedOutputCore
    + BufferKindUser<BufferKind=Eager>
    + Default
    + Clone,
    <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
    Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero,
{
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            _marker: std::marker::PhantomData,
        }
    }
    
    pub async fn new_from_key_file(key_file: &str) -> Result<Self> {
        let key = tokio::fs::read(key_file).await
            .context(format!("Failed to read key file: {}", key_file))?;
        Ok(Self::new(key))
    }
}


impl<D> Signer for HmacSigner<D>
where
    D: CoreProxy,
    D::Core: HashMarker
    + UpdateCore
    + FixedOutputCore
    + BufferKindUser<BufferKind=Eager>
    + Default
    + Clone,
    <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
    Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero,
{
    fn sign(&self, data: &[u8]) -> Result<Bytes> {
        let mut hmac: Hmac<D> = Hmac::new_from_slice(&self.key)
            .context("Failed to create hmac")?;
        hmac.update(&data);
        let signature = hmac.finalize().into_bytes().to_vec();
        let mut signed_data = BytesMut::new();
        signed_data.extend_from_slice(&data);
        signed_data.extend_from_slice(&signature);
        Ok(signed_data.freeze())
    }

    fn verify(&self, data: &[u8]) -> Result<Bytes> {
        let data_len = data.len();
        let signature_len = Sha256::output_size();
        if data_len < signature_len {
            return Err(anyhow::anyhow!("Invalid data length"));
        }
        let (data, signature) = data.split_at(data_len - signature_len);
        let mut hmac: Hmac<D> = Hmac::new_from_slice(&self.key)
            .context("Failed to create hmac")?;
        hmac.update(data);
        hmac.verify_slice(signature).context("Failed to verify signature")?;
        Ok(Bytes::copy_from_slice(data))
    }
}

pub struct UnsecureSigner;

impl Signer for UnsecureSigner {
    fn sign(&self, data: &[u8]) -> Result<Bytes> {
        Ok(Bytes::copy_from_slice(data))
    }

    fn verify(&self, data: &[u8]) -> Result<Bytes> {
        Ok(Bytes::copy_from_slice(data))
    }
}