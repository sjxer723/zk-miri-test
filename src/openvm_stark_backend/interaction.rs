use crate::openvm_stark_backend::air::AirBuilder;

// Please refer to
// https://github.com/openvm-org/stark-backend/blob/main/crates/stark-backend/src/interaction/mod.rs
// for full implementation details.

pub type BusIndex = usize;

#[derive(Clone, PartialEq, Eq)]
pub struct Interaction<Expr> {
    pub message: Vec<Expr>,
    pub count: Expr,
    /// The bus index specifying the bus to send the message over. All valid instantiations of
    /// `BusIndex` are safe.
    pub bus_index: BusIndex,
    /// Determines the contribution of each interaction message to a linear constraint on the trace
    /// heights in the verifier.
    ///
    /// For each bus index and trace, `count_weight` values are summed per interaction on that
    /// bus index and multiplied by the trace height. The total sum over all traces is constrained
    /// by the verifier to not overflow the field characteristic \( p \).
    ///
    /// This is used to impose sufficient conditions for bus constraint soundness and setting a
    /// proper value depends on the bus and the constraint it imposes.
    pub count_weight: u32,
}

pub trait InteractionBuilder: AirBuilder {
    /// Stores a new interaction in the builder.
    ///
    /// See [Interaction] for more details on `count_weight`.
    fn push_interaction<E: Into<Self::Expr>>(
        &mut self,
        bus_index: BusIndex,
        fields: impl IntoIterator<Item = E>,
        count: impl Into<Self::Expr>,
        count_weight: u32,
    );

    // Returns the current number of interactions.
    // fn num_interactions(&self) -> usize;

    // Returns all interactions stored.
    // fn all_interactions(&self) -> &[Interaction<Self::Expr>];
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LookupBus {
    pub index: BusIndex,
}

impl LookupBus {
    pub const fn new(index: BusIndex) -> Self {
        Self { index }
    }

    /// Performs a lookup on the given bus.
    ///
    /// This method asserts that `key` is present in the lookup table. The parameter `enabled`
    /// must be constrained to be boolean, and the lookup constraint is imposed provided `enabled`
    /// is one.
    ///
    /// Caller must constrain that `enabled` is boolean.
    pub fn lookup_key<AB, E>(
        &self,
        builder: &mut AB,
        query: impl IntoIterator<Item = E>,
        enabled: impl Into<AB::Expr>,
    ) where
        AB: InteractionBuilder,
        E: Into<AB::Expr>,
    {
        // We embed the query multiplicity as {0, 1} in the integers and the lookup table key
        // multiplicity to be {0, -1, ..., -p + 1}. Setting `count_weight = 1` will ensure that the
        // total number of lookups is at most p, which is sufficient to establish lookup multiset is
        // a subset of the key multiset. See Corollary 3.6 in
        // [docs/Soundess_of_Interactions_via_LogUp.pdf].
        builder.push_interaction(self.index, query, enabled, 1);
    }

    /// Adds a key to the lookup table.
    ///
    /// The `num_lookups` parameter should equal the number of enabled lookups performed.

    pub fn add_key_with_lookups<AB, E>(
        &self,
        builder: &mut AB,
        key: impl IntoIterator<Item = E>,
        num_lookups: impl Into<AB::Expr>,
    ) where
        AB: InteractionBuilder,
        E: Into<AB::Expr>,
    {
        // Since we only want a subset constraint, `count_weight` can be zero here. See the comment
        // in `LookupBus::lookup_key`.
        builder.push_interaction(self.index, key, -num_lookups.into(), 0);
    }
}
