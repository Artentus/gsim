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

pub(crate) struct Wire {
    state_id: WireStateId,
    drivers: IdVec<OutputStateId>,
    driving: IdVec<ComponentId>,
}

impl Wire {
    #[inline]
    pub(crate) fn new(state_id: WireStateId) -> Self {
        Self {
            state_id,
            drivers: IdVec::new(),
            driving: IdVec::new(),
        }
    }

    #[inline]
    pub(crate) fn state_id(&self) -> WireStateId {
        self.state_id
    }

    #[inline]
    pub(crate) fn drivers(&self) -> &[OutputStateId] {
        self.drivers.as_slice()
    }

    #[inline]
    pub(crate) fn driving(&self) -> &[ComponentId] {
        self.driving.as_slice()
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
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_0 | O_1 |     O     | conflict
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

    let plane_0 = a[1] & b[1];
    let plane_1 = a[0] | b[0];
    let conflict = (!a[1] & !b[1]) | (!a[1] & b[0]) | (a[0] & !b[1]) | (a[0] & b[0]);
    ([plane_0, plane_1], conflict)
}

impl Wire {
    #[inline]
    pub(crate) fn update(
        &self,
        mut state: LogicStateMut,
        drive: LogicStateRef,
        output_states: OutputStateView,
    ) -> WireUpdateResult {
        assert_eq!(state.bit_width(), drive.bit_width());
        let bit_width = state.bit_width();

        let (state_plane_0, state_plane_1) = state.bit_planes_mut();
        let (drive_plane_0, drive_plane_1) = drive.bit_planes();
        assert_eq!(state_plane_0.len(), state_plane_1.len());
        assert_eq!(drive_plane_0.len(), drive_plane_1.len());
        assert_eq!(state_plane_0.len(), drive_plane_0.len());
        let word_len = state_plane_0.len();

        let mut tmp_plane_0 = [0; MAX_WORD_COUNT];
        let mut tmp_plane_1 = [0; MAX_WORD_COUNT];

        {
            let tmp_plane_0 = &mut tmp_plane_0[..word_len];
            let tmp_plane_1 = &mut tmp_plane_1[..word_len];
            tmp_plane_0.copy_from_slice(drive_plane_0);
            tmp_plane_1.copy_from_slice(drive_plane_1);
        }

        let mut conflict = 0;
        for driver in self.drivers.iter() {
            let [driver_state] = output_states.get(driver).expect("invalid output state ID");
            assert_eq!(bit_width, driver_state.bit_width());

            let (driver_plane_0, driver_plane_1) = driver_state.bit_planes();
            assert_eq!(driver_plane_0.len(), driver_plane_1.len());
            assert_eq!(state_plane_0.len(), driver_plane_0.len());

            let tmp_plane_0 = &mut tmp_plane_0[..word_len];
            let tmp_plane_1 = &mut tmp_plane_1[..word_len];

            for (i, (tmp_word_0, tmp_word_1, &driver_word_0, &driver_word_1)) in
                izip!(tmp_plane_0, tmp_plane_1, driver_plane_0, driver_plane_1).enumerate()
            {
                let ([new_word_0, new_word_1], new_conflict) =
                    combine([*tmp_word_0, *tmp_word_1], [driver_word_0, driver_word_1]);

                let mask = if i == (word_len - 1) {
                    bit_width.last_word_mask()
                } else {
                    u32::MAX
                };

                *tmp_word_0 = new_word_0 & mask;
                *tmp_word_1 = new_word_1 & mask;
                conflict |= new_conflict & mask;
            }
        }

        let mut changed = false;
        {
            let tmp_plane_0 = &tmp_plane_0[..word_len];
            let tmp_plane_1 = &tmp_plane_1[..word_len];

            for (state_word_0, state_word_1, &tmp_word_0, &tmp_word_1) in
                izip!(state_plane_0, state_plane_1, tmp_plane_0, tmp_plane_1)
            {
                changed |= (tmp_word_0 != *state_word_0) | (tmp_word_1 != *state_word_1);
                *state_word_0 = tmp_word_0;
                *state_word_1 = tmp_word_1;
            }
        }

        if conflict != 0 {
            WireUpdateResult::Conflict
        } else if changed {
            WireUpdateResult::Changed
        } else {
            WireUpdateResult::Unchanged
        }
    }
}
