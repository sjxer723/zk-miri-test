use crate::openvm_stark_backend::{
    field::{F, FieldAlgebra},
    interaction::{BusIndex, InteractionBuilder, LookupBus},
};

// Please refer to
// https://github.com/openvm-org/openvm/blob/3c800070d363237832a66dbe5501d3c365f3c549/crates/circuits/primitives/src/bitwise_op_lookup/bus.rs#L83
// for full implementation details.

#[derive(Clone)]
pub struct BitwiseOperationLookupBusInteraction<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub op: T,
    pub bus: LookupBus,
    is_lookup: bool,
}

impl<T: FieldAlgebra> BitwiseOperationLookupBusInteraction<T> {
    pub fn eval<AB>(self, builder: &mut AB, count: impl Into<AB::Expr>)
    where
        AB: InteractionBuilder<Expr = T>,
    {
        let key = [self.x, self.y, self.z, self.op];
        if self.is_lookup {
            self.bus.lookup_key(builder, key, count);
        } else {
            self.bus.add_key_with_lookups(builder, key, count);
        }
    }
}

#[derive(Clone)]
pub struct BitwiseOperationLookupBus {
    pub inner: LookupBus,
}

impl BitwiseOperationLookupBus {
    pub const fn new(index: BusIndex) -> Self {
        Self { inner: LookupBus::new(index) }
    }

    #[must_use]
    pub fn send_range<T>(
        &self,
        x: impl Into<T>,
        y: impl Into<T>,
    ) -> BitwiseOperationLookupBusInteraction<T>
    where
        T: FieldAlgebra,
    {
        self.push(x, y, T::ZERO, T::ZERO, true)
    }

    #[must_use]
    pub fn send_xor<T>(
        &self,
        x: impl Into<T>,
        y: impl Into<T>,
        z: impl Into<T>,
    ) -> BitwiseOperationLookupBusInteraction<T>
    where
        T: FieldAlgebra,
    {
        self.push(x, y, z, T::ONE, true)
    }

    #[must_use]
    pub fn receive<T>(
        &self,
        x: impl Into<T>,
        y: impl Into<T>,
        z: impl Into<T>,
        op: impl Into<T>,
    ) -> BitwiseOperationLookupBusInteraction<T> {
        self.push(x, y, z, op, false)
    }

    pub fn push<T>(
        &self,
        x: impl Into<T>,
        y: impl Into<T>,
        z: impl Into<T>,
        op: impl Into<T>,
        is_lookup: bool,
    ) -> BitwiseOperationLookupBusInteraction<T> {
        BitwiseOperationLookupBusInteraction {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            op: op.into(),
            bus: self.inner,
            is_lookup,
        }
    }
}
