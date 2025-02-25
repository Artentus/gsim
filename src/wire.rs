use crate::component::ComponentId;
use crate::id::*;
use crate::logic::*;
use itertools::izip;

def_id_type!(
    /// A unique identifier for a wire inside a simulation
    pub WireId
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub(crate) enum WireUpdateResult {
    Unchanged,
    Changed,
    Conflict,
}

impl From<CopyFromResult> for WireUpdateResult {
    #[inline]
    fn from(value: CopyFromResult) -> Self {
        match value {
            CopyFromResult::Unchanged => Self::Unchanged,
            CopyFromResult::Changed => Self::Changed,
        }
    }
}

pub(crate) struct Wire {
    bit_width: BitWidth,
    state_id: WireStateId,
    drivers: IdVec<OutputStateId>,
    driving: IdVec<ComponentId>,
}

impl Wire {
    #[inline]
    pub(crate) fn new(bit_width: BitWidth, state_id: WireStateId) -> Self {
        Self {
            bit_width,
            state_id,
            drivers: IdVec::new(),
            driving: IdVec::new(),
        }
    }

    #[inline]
    pub(crate) fn bit_width(&self) -> BitWidth {
        self.bit_width
    }

    #[inline]
    pub(crate) fn state_id(&self) -> WireStateId {
        self.state_id
    }

    #[inline]
    pub(crate) fn drivers(&self) -> &[OutputStateId] {
        &self.drivers
    }

    #[inline]
    pub(crate) fn driving(&self) -> &[ComponentId] {
        &self.driving
    }

    pub(crate) fn add_driver(&mut self, output: OutputStateId) {
        self.drivers.push(output);
    }

    pub(crate) fn add_driving(&mut self, component: ComponentId) {
        // This is a linear search which may appear slow, but the list is usually very small so the overhead
        // of a hashset is not actually worth it.
        // In particular, the lookup only occurs while building the graph, whereas during simulation, when speed
        // is important, reading a vector is much faster than reading a hashset.
        if !self.driving.contains(component) {
            self.driving.push(component);
        }
    }
}

#[inline]
fn combine(a: [u32; 2], b: [u32; 2]) -> ([u32; 2], u32) {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O     | conflict
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------|----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  -  |  -  | -         | yes
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  -  |  -  | -         | yes
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  0  |  0  | Logic 0   | no
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  -  |  -  | -         | yes
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------|----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  -  |  -  | -         | yes
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  -  |  -  | -         | yes
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  0  |  1  | Logic 1   | no
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  -  |  -  | -         | yes
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------|----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  0  |  0  | Logic 0   | no
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  0  |  1  | Logic 1   | no
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  0  | High-Z    | no
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined | no
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------|----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  -  |  -  | -         | yes
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  -  |  -  | -         | yes
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined | no
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  -  |  -  | -         | yes

    let plane_0 = a[0] | b[0];
    let plane_1 = a[1] & b[1];
    let conflict = (!a[1] & !b[1]) | (!a[1] & b[0]) | (a[0] & !b[1]) | (a[0] & b[0]);
    ([plane_0, plane_1], conflict)
}

impl Wire {
    #[inline]
    pub(crate) fn update(
        &self,
        mut wire_states: WireStateViewMut,
        output_states: OutputStateView,
    ) -> WireUpdateResult {
        let [mut state, drive] = wire_states
            .get_mut(self.state_id, self.bit_width)
            .expect("invalid wire state ID");
        let word_len = self.bit_width.word_len() as usize;

        let mut tmp_state = InlineLogicState::logic_0(self.bit_width);
        tmp_state.copy_from(drive);

        let mut conflict = 0;
        for driver in self.drivers.iter() {
            let [driver_state] = output_states
                .get(driver, self.bit_width)
                .expect("invalid output state ID");
            let (driver_plane_0, driver_plane_1) = driver_state.bit_planes();
            let (tmp_plane_0, tmp_plane_1) = tmp_state.bit_planes_mut();

            for (i, (tmp_word_0, tmp_word_1, &driver_word_0, &driver_word_1)) in
                izip!(tmp_plane_0, tmp_plane_1, driver_plane_0, driver_plane_1).enumerate()
            {
                let ([new_word_0, new_word_1], new_conflict) =
                    combine([*tmp_word_0, *tmp_word_1], [driver_word_0, driver_word_1]);

                let mask = if i == (word_len - 1) {
                    self.bit_width.last_word_mask()
                } else {
                    u32::MAX
                };

                *tmp_word_0 = new_word_0 & mask;
                *tmp_word_1 = new_word_1 & mask;
                conflict |= new_conflict & mask;
            }
        }

        let copy_result = state.copy_from(&tmp_state);

        if conflict != 0 {
            WireUpdateResult::Conflict
        } else {
            copy_result.into()
        }
    }
}

def_id_list!(WireList<WireId, Wire>);

impl WireList {
    #[inline]
    pub(crate) fn wire_count(&self) -> usize {
        self.0.len()
    }
}
