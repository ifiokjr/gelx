# gelx_core

<br />

> This crate contains the core logic for the `gelx` crate.

## Code generation

Code generation is available for both queries and the database schema.

With the following schema:

```gel
module default {
  type Person {
    required name: str;
  }

  scalar type Genre extending enum<Horror, Comedy, Drama>;

  type Movie {
    required title: str;
    genre: Genre;
    multi actors: Person;
  }
}

module alternative {
	scalar type Awesomeness extending enum<Very, Somewhat, Not>;
}
```

The following generated code is produced:

```rust
use ::gelx::exports as e;

use
#[derive(
	Debug,
	Clone,
	Copy,
	e::serde::Serialize,
	e::serde::Deserialize,
	e::gel_derive::Queryable,
	strum::AsRefStr,
	strum::Display,
	strum::EnumString,
	strum::EnumIs,
	strum::FromRepr,
	strum::IntoStaticStr,
)]
pub enum Genre {
	Horror,
	Comedy,
	Drama,
}

pub mod alternative {
	use super::*;

	#[derive(
		Debug,
		Clone,
		Copy,
		e::serde::Serialize,
		e::serde::Deserialize,
		e::gel_derive::Queryable,
		strum::AsRefStr,
		strum::Display,
		strum::EnumString,
		strum::EnumIs,
		strum::FromRepr,
		strum::IntoStaticStr,
	)]
	pub enum Awesomeness {
		Very,
		Somewhat,
		Not,
	}
}
```
