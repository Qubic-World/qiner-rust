use crate::types::{Id, PublicKey64};

const A: u8 = 'A' as u8;

pub fn get_public_key_64_from_id(id: &Id, public_key: &mut PublicKey64) -> bool {
	*public_key = Default::default();

	for i in 0..4 {
		for j in (0..14).rev() {
			let id_value = id[i * 14 + j];
			if id_value < 'A' as u8 || id_value > 'Z' as u8 {
				*public_key = Default::default();

				return false;
			}

			let delta_id_value = (id_value - A) as u64;

			public_key[i] = public_key[i] * 26u64 + delta_id_value;
		}
	}

	true
}
