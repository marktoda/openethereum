// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use log::*;
use parity_crypto::publickey::Secret;
use parity_path::restrict_permissions_owner;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// An entity that can be persisted on disk.
pub trait DiskEntity: Sized {
	const FILENAME: &'static str;
	/// Description of what kind of data that is stored in the file
	const DESCRIPTION: &'static str;

	/// Convert to UTF-8 representation that will be written to disk.
	fn to_repr(&self) -> String;

	/// Convert from UTF-8 representation loaded from disk.
	fn from_repr(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>;
}

impl DiskEntity for Secret {
	const FILENAME: &'static str = "key";
	const DESCRIPTION: &'static str = "key file";

	fn to_repr(&self) -> String {
		self.to_hex()
	}

	fn from_repr(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
		Ok(s.parse()?)
	}
}

pub fn save<E: DiskEntity>(path: &Path, entity: &E) {
	let mut path_buf = PathBuf::from(path);
	if let Err(e) = fs::create_dir_all(path_buf.as_path()) {
		warn!("Error creating {} directory: {:?}", E::DESCRIPTION, e);
		return;
	};
	path_buf.push(E::FILENAME);
	let path = path_buf.as_path();
	let mut file = match fs::File::create(&path) {
		Ok(file) => file,
		Err(e) => {
			warn!("Error creating {}: {:?}", E::DESCRIPTION, e);
			return;
		}
	};
	if let Err(e) = restrict_permissions_owner(path, true, false) {
		warn!(target: "network", "Failed to modify permissions of the file ({})", e);
	}
	if let Err(e) = file.write(&entity.to_repr().into_bytes()) {
		warn!("Error writing {}: {:?}", E::DESCRIPTION, e);
	}
}

pub fn load<E>(path: &Path) -> Option<E>
where
	E: DiskEntity,
{
	let mut path_buf = PathBuf::from(path);
	path_buf.push(E::FILENAME);
	let mut file = match fs::File::open(path_buf.as_path()) {
		Ok(file) => file,
		Err(e) => {
			debug!("Error opening {}: {:?}", E::DESCRIPTION, e);
			return None;
		}
	};
	let mut buf = String::new();
	match file.read_to_string(&mut buf) {
		Ok(_) => {},
		Err(e) => {
			warn!("Error reading {}: {:?}", E::DESCRIPTION, e);
			return None;
		}
	}
	match E::from_repr(&buf) {
		Ok(key) => Some(key),
		Err(e) => {
			warn!("Error parsing {}: {:?}", E::DESCRIPTION, e);
			None
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn key_save_load() {
		use super::*;
		use ethereum_types::H256;
		use tempdir::TempDir;

		let tempdir = TempDir::new("").unwrap();
		let key = Secret::from(H256::random());
		save(tempdir.path(), &key);
		let r = load(tempdir.path());
		assert_eq!(key, r.unwrap());
	}
}