use bitcoin::{Transaction, Weight};
use brk_types::{TxIndex, Txid, TxidPrefix, Weight as TypesWeight};

pub struct ComputedTx<'a> {
    pub tx_index: TxIndex,
    pub tx: &'a Transaction,
    pub txid: Txid,
    pub base_size: u32,
    pub total_size: u32,
    pub input_offset: usize,
    pub output_offset: usize,
}

impl ComputedTx<'_> {
    pub fn new(
        tx_index: TxIndex,
        tx: &Transaction,
        txid: Txid,
        base_size: u32,
        total_size: u32,
    ) -> ComputedTx<'_> {
        ComputedTx {
            tx_index,
            tx,
            txid,
            base_size,
            total_size,
            input_offset: 0,
            output_offset: 0,
        }
    }

    pub fn set_block_offsets(txs: &mut [Self]) {
        let mut input_offset = 0;
        let mut output_offset = 0;

        for tx in txs {
            tx.input_offset = input_offset;
            tx.output_offset = output_offset;
            input_offset += tx.tx.input.len();
            output_offset += tx.tx.output.len();
        }
    }

    #[inline]
    pub fn inputs<'a, T>(&self, inputs: &'a [T]) -> &'a [T] {
        &inputs[self.input_offset..self.input_offset + self.tx.input.len()]
    }

    #[inline]
    pub fn outputs<'a, T>(&self, outputs: &'a [T]) -> &'a [T] {
        &outputs[self.output_offset..self.output_offset + self.tx.output.len()]
    }

    #[inline]
    pub fn txout_offset(&self, vout: u32) -> usize {
        self.output_offset + vout as usize
    }

    #[inline]
    pub fn is_coinbase(&self, block_first_tx_index: TxIndex) -> bool {
        self.tx_index == block_first_tx_index
    }

    #[inline]
    pub fn txid_prefix(&self) -> TxidPrefix {
        TxidPrefix::from(&self.txid)
    }

    #[inline]
    pub fn is_segwit(&self) -> bool {
        self.base_size != self.total_size
    }

    #[inline]
    pub fn weight(&self) -> Weight {
        TypesWeight::from_sizes(self.base_size, self.total_size).into()
    }
}
