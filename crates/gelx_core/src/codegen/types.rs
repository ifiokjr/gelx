use std::collections::HashMap;

use bitflags::bitflags;
use gel_protocol::common::Cardinality;
use gel_tokio::Client;
use gel_tokio::Queryable;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::GelxCoreResult;

/// Execute the types query to get the types of the current database.
pub(crate) async fn query_types(client: &Client) -> GelxCoreResult<Vec<TypesOutput>> {
	let result = client.query(TYPES_QUERY, &()).await?;

	Ok(result)
}

#[derive(Clone, Debug, Queryable)]
pub struct BasesSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable)]
pub struct UnionOfSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable)]
pub struct IntersectionOfSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable)]
pub struct PointersSetPointersSet {
	pub card: Option<String>,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_computed: Option<bool>,
	pub is_readonly: Option<bool>,
}

impl PointersSetPointersSet {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(self.card.as_deref())
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct PointersSet {
	pub card: Option<String>,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: bool,
	pub is_computed: Option<bool>,
	pub is_readonly: Option<bool>,
	pub has_default: bool,
	pub pointers: Vec<PointersSetPointersSet>,
}

impl PointersSet {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(self.card.as_deref())
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct ExclusivesSet {
	pub target: Option<String>,
}

#[derive(Clone, Debug, Queryable)]
pub struct BacklinksSet {
	pub card: String,
	pub name: String,
	pub stub: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: Option<bool>,
}

impl BacklinksSet {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(Some(&self.card))
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct BacklinkStubsArray {
	pub card: String,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: bool,
}

impl BacklinkStubsArray {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(Some(&self.card))
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct TupleElementsSet {
	pub target_id: Uuid,
	pub name: Option<String>,
}

#[derive(Clone, Debug, Queryable)]
pub struct TypesOutput {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: Option<bool>,
	pub kind: String,
	pub enum_values: Option<Vec<String>>,
	pub is_seq: bool,
	pub material_id: Option<Uuid>,
	pub bases: Vec<BasesSet>,
	pub union_of: Vec<UnionOfSet>,
	pub intersection_of: Vec<IntersectionOfSet>,
	pub pointers: Vec<PointersSet>,
	pub exclusives: Vec<ExclusivesSet>,
	pub backlinks: Vec<BacklinksSet>,
	pub backlink_stubs: Vec<BacklinkStubsArray>,
	pub array_element_id: Option<Uuid>,
	pub tuple_elements: Vec<TupleElementsSet>,
	pub multirange_element_id: Option<Uuid>,
}

impl TypesOutput {
	pub(crate) fn pointers_map(&self) -> HashMap<String, Pointer> {
		self.pointers
			.iter()
			.map(|p| (p.name.clone(), p.clone().into()))
			.collect()
	}

	pub fn exclusives(&self, pointers: &HashMap<String, Pointer>) -> Vec<Exclusives> {
		let mut exclusives = Vec::new();
		for exclusive in &self.exclusives {
			let Some(target) = &exclusive.target else {
				continue;
			};

			if let Some(pointer) = pointers.get(target) {
				exclusives.push(Exclusives::One(pointer.clone()));
				continue;
			}

			if target.starts_with('(') && target.ends_with(')') {
				// Handle multiple targets case
				let targets = target
					.trim_matches(|c| c == '(' || c == ')')
					.split(' ')
					.map(|t| {
						t.trim()
							.trim_start_matches('.')
							.trim_end_matches(',')
							.to_string()
					})
					.collect::<Vec<_>>();

				let mut target_pointers = vec![];

				for target in &targets {
					if let Some(pointer) = pointers.get(target) {
						target_pointers.push(pointer.clone());
					}
				}

				if target_pointers.is_empty() {
					continue;
				}

				if target_pointers.len() == 1 {
					exclusives.push(Exclusives::One(target_pointers[0].clone()));
				} else {
					exclusives.push(Exclusives::Many(target_pointers));
				}
			}
		}

		exclusives
	}

	pub fn backlinks(&self) -> Vec<Backlink> {
		let re = regex::Regex::new(r"\[is (.+)\]").unwrap();
		let mut backlinks = Vec::new();

		for backlink in &self.backlinks {
			let Some(target_id) = backlink.target_id else {
				continue;
			};

			let Some(matched_name) = re.captures(&backlink.name).and_then(|c| c.get(1)) else {
				continue;
			};

			let mut new_backlink = Backlink {
				cardinality: backlink.cardinality(),
				name: backlink.name.clone(),
				target_id,
				is_exclusive: backlink.is_exclusive.unwrap_or_default(),
				stub: Some(backlink.stub.clone()),
			};

			let Some((module_name, local_name)) = matched_name.as_str().split_once("::") else {
				backlinks.push(new_backlink);
				continue;
			};

			if module_name != "default" {
				backlinks.push(new_backlink);
				continue;
			}

			new_backlink.name = re
				.replace(&new_backlink.name, |_: &regex::Captures| {
					format!("[is {local_name}]")
				})
				.to_string();

			backlinks.push(new_backlink);
		}

		for backlink in &self.backlink_stubs {
			let Some(target_id) = backlink.target_id else {
				continue;
			};

			let Some(captures) = re.captures(&backlink.name) else {
				continue;
			};

			let Some(matched_name) = captures.get(1) else {
				continue;
			};

			let mut new_backlink = Backlink {
				cardinality: backlink.cardinality(),
				name: backlink.name.clone(),
				target_id,
				is_exclusive: backlink.is_exclusive,
				stub: None,
			};

			let Some((module_name, local_name)) = matched_name.as_str().split_once("::") else {
				backlinks.push(new_backlink);
				continue;
			};

			if module_name != "default" {
				backlinks.push(new_backlink);
				continue;
			}

			new_backlink.name = re
				.replace(&new_backlink.name, |_: &regex::Captures| {
					format!("[is {local_name}]")
				})
				.to_string();

			backlinks.push(new_backlink);
		}

		backlinks
	}
}

pub(crate) fn to_cardinality(cardinality: Option<&str>) -> Cardinality {
	let Some(cardinality) = cardinality else {
		return Cardinality::NoResult;
	};

	match cardinality {
		"AtMostOne" => Cardinality::AtMostOne,
		"One" => Cardinality::One,
		"Many" => Cardinality::Many,
		"AtLeastOne" => Cardinality::AtLeastOne,
		_ => Cardinality::NoResult,
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerKind {
	Link,
	Property,
}

bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct PointerFlags: u8 {
		const IS_EXCLUSIVE = 0b0000_0001;
		const IS_COMPUTED = 0b0000_0010;
		const IS_READONLY = 0b0000_0100;
		const HAS_DEFAULT = 0b0000_1000;
	}
}

#[derive(Debug, Clone)]
pub struct Pointer {
	pub card: Cardinality,
	pub kind: PointerKind,
	pub name: String,
	pub target_id: Uuid,
	pub flags: PointerFlags,
	pub pointers: Option<Vec<Pointer>>,
}

impl From<PointersSet> for Pointer {
	fn from(value: PointersSet) -> Self {
		let mut flags = PointerFlags::empty();

		if value.is_exclusive {
			flags.insert(PointerFlags::IS_EXCLUSIVE);
		}

		if value.is_computed.unwrap_or_default() {
			flags.insert(PointerFlags::IS_COMPUTED);
		}

		if value.is_readonly.unwrap_or_default() {
			flags.insert(PointerFlags::IS_READONLY);
		}

		if value.has_default {
			flags.insert(PointerFlags::HAS_DEFAULT);
		}

		Pointer {
			card: value.cardinality(),
			kind: match value.kind.as_str() {
				"link" => PointerKind::Link,
				"property" => PointerKind::Property,
				_ => panic!("Invalid pointer kind: {}", value.kind),
			},
			name: value.name,
			target_id: value.target_id.unwrap(),
			flags,
			pointers: Some(value.pointers.iter().map(|p| p.clone().into()).collect()),
		}
	}
}

impl From<PointersSetPointersSet> for Pointer {
	fn from(value: PointersSetPointersSet) -> Self {
		let mut flags = PointerFlags::empty();

		if value.is_computed.unwrap_or_default() {
			flags.insert(PointerFlags::IS_COMPUTED);
		}

		if value.is_readonly.unwrap_or_default() {
			flags.insert(PointerFlags::IS_READONLY);
		}

		Pointer {
			card: value.cardinality(),
			kind: match value.kind.as_str() {
				"link" => PointerKind::Link,
				"property" => PointerKind::Property,
				_ => panic!("Invalid pointer kind: {}", value.kind),
			},
			name: value.name,
			target_id: value.target_id.unwrap(),
			flags,
			pointers: None,
		}
	}
}

impl Pointer {
	pub fn is_link(&self) -> bool {
		self.kind == PointerKind::Link
	}

	pub fn is_property(&self) -> bool {
		self.kind == PointerKind::Property
	}

	pub fn is_exclusive(&self) -> bool {
		self.flags.contains(PointerFlags::IS_EXCLUSIVE)
	}

	pub fn is_computed(&self) -> bool {
		self.flags.contains(PointerFlags::IS_COMPUTED)
	}

	pub fn has_default(&self) -> bool {
		self.flags.contains(PointerFlags::HAS_DEFAULT)
	}

	pub fn is_readonly(&self) -> bool {
		self.flags.contains(PointerFlags::IS_READONLY)
	}
}

#[derive(Debug, Clone)]
pub struct Backlink {
	// Fields from Pointer without flags, kind, pointers
	pub cardinality: Cardinality,
	pub name: String,
	pub target_id: Uuid,

	// Specific to Backlink
	pub is_exclusive: bool,
	pub stub: Option<String>,
}

impl Backlink {
	pub fn is_stub(&self) -> bool {
		self.stub.is_none()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
	Object,
	Scalar,
	Array,
	Tuple,
	Range,
	MultiRange,
}

// Helper struct for fields like `bases: readonly { id: UUID }[]`
#[derive(Debug, Clone)]
pub struct IdRef {
	pub id: Uuid,
}

impl From<BasesSet> for IdRef {
	fn from(value: BasesSet) -> Self {
		IdRef { id: value.id }
	}
}

impl From<UnionOfSet> for IdRef {
	fn from(value: UnionOfSet) -> Self {
		IdRef { id: value.id }
	}
}

impl From<IntersectionOfSet> for IdRef {
	fn from(value: IntersectionOfSet) -> Self {
		IdRef { id: value.id }
	}
}

// Structs for each type kind. Note: 'kind' field is handled by the Type enum
// tag.
#[derive(Debug, Clone)]
pub struct ScalarType {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: bool,
	pub is_seq: bool,
	pub bases: Vec<IdRef>,
	pub material_id: Option<Uuid>,
	pub cast_type: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct EnumType {
	pub id: Uuid,
	pub name: String,
	pub enum_values: Vec<String>,
	pub bases: Vec<IdRef>,
}

#[derive(Clone, Debug)]
pub enum Exclusives {
	One(Pointer),
	Many(Vec<Pointer>),
}

#[derive(Debug, Clone)]
pub struct ObjectType {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: bool,
	pub bases: Vec<IdRef>,
	pub union_of: Vec<IdRef>,
	pub intersection_of: Vec<IdRef>,
	pub pointers: Vec<Pointer>,
	pub backlinks: Vec<Backlink>,
	pub exclusives: Vec<Exclusives>,
}

#[derive(Debug, Clone)]
pub struct ArrayType {
	pub id: Uuid,
	pub bases: Vec<IdRef>,
	pub name: String,
	pub array_element_id: Uuid,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct TupleElementDef {
	pub name: String,
	pub target_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct TupleType {
	pub id: Uuid,
	pub name: String,
	pub tuple_elements: Vec<TupleElementDef>,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct RangeType {
	pub id: Uuid,
	pub name: String,
	pub range_element_id: Uuid,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct MultiRangeType {
	pub id: Uuid,
	pub name: String,
	pub multirange_element_id: Uuid,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct BaseType {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: bool, // In TS, this is specified as 'false' for BaseType
}

#[derive(Debug, Clone)]
pub enum Type {
	Object(ObjectType),
	Scalar(ScalarType),
	Enum(EnumType),
	Array(ArrayType),
	Tuple(TupleType),
	Range(RangeType),
	MultiRange(MultiRangeType),
	Base(BaseType),
}

impl Type {
	pub fn id(&self) -> Uuid {
		match self {
			Type::Object(obj) => obj.id,
			Type::Scalar(scalar) => scalar.id,
			Type::Enum(enum_type) => enum_type.id,
			Type::Array(array_type) => array_type.id,
			Type::Tuple(tuple_type) => tuple_type.id,
			Type::Range(range_type) => range_type.id,
			Type::MultiRange(multi_range_type) => multi_range_type.id,
			Type::Base(base_type) => base_type.id,
		}
	}

	pub fn name(&self) -> &str {
		match self {
			Type::Object(obj) => &obj.name,
			Type::Scalar(scalar) => &scalar.name,
			Type::Enum(enum_type) => &enum_type.name,
			Type::Array(array_type) => &array_type.name,
			Type::Tuple(tuple_type) => &tuple_type.name,
			Type::Range(range_type) => &range_type.name,
			Type::MultiRange(multi_range_type) => &multi_range_type.name,
			Type::Base(base_type) => &base_type.name,
		}
	}

	pub fn is_primitive(&self) -> bool {
		matches!(
			self,
			Type::Scalar(_)
				| Type::Array(_)
				| Type::Tuple(_)
				| Type::Range(_)
				| Type::MultiRange(_)
		)
	}
}

pub type Types = IndexMap<Uuid, Type>;

pub(crate) fn map_fetched_types(fetched_types: &[TypesOutput]) -> Types {
	let mut types = IndexMap::new();

	for type_info in fetched_types {
		match type_info.kind.as_str() {
			"scalar" => {
				if let Some(enum_values) = &type_info.enum_values {
					let enum_type = EnumType {
						id: type_info.id,
						name: type_info.name.clone(),
						enum_values: enum_values.clone(),
						bases: type_info
							.bases
							.iter()
							.map(|base| base.clone().into())
							.collect(),
					};
					types.insert(type_info.id, Type::Enum(enum_type));
				} else {
					let scalar_type = ScalarType {
						id: type_info.id,
						name: type_info.name.clone(),
						is_abstract: type_info.is_abstract.unwrap_or_default(),
						is_seq: type_info.is_seq,
						bases: type_info
							.bases
							.iter()
							.map(|base| base.clone().into())
							.collect(),
						material_id: type_info.material_id,
						// TODO: doesn't seem useful in rust
						cast_type: None,
					};
					types.insert(type_info.id, Type::Scalar(scalar_type));
				}
			}
			"range" => {
				let range_type = RangeType {
					id: type_info.id,
					name: type_info.name.clone(),
					range_element_id: type_info.multirange_element_id.unwrap(),
					is_abstract: type_info.is_abstract.unwrap_or_default(),
				};
				types.insert(type_info.id, Type::Range(range_type));
			}
			"multirange" => {
				let multirange_type = MultiRangeType {
					id: type_info.id,
					name: type_info.name.clone(),
					multirange_element_id: type_info.multirange_element_id.unwrap(),
					is_abstract: type_info.is_abstract.unwrap_or_default(),
				};
				types.insert(type_info.id, Type::MultiRange(multirange_type));
			}
			"object" => {
				let pointers = type_info.pointers_map();
				let exclusives = type_info.exclusives(&pointers);
				let object_type = ObjectType {
					id: type_info.id,
					name: type_info.name.clone(),
					is_abstract: type_info.is_abstract.unwrap_or_default(),
					bases: type_info
						.bases
						.iter()
						.map(|base| base.clone().into())
						.collect(),
					union_of: type_info
						.union_of
						.iter()
						.map(|u| u.clone().into())
						.collect(),
					intersection_of: type_info
						.intersection_of
						.iter()
						.map(|i| i.clone().into())
						.collect(),
					pointers: type_info
						.pointers
						.iter()
						.map(|p| p.clone().into())
						.collect(),
					backlinks: type_info.backlinks(),
					exclusives,
				};
				types.insert(type_info.id, Type::Object(object_type));
			}
			_ => {}
		}
	}

	types
}

/// Query to get all types in the database.
pub const TYPES_QUERY: &str = r#"WITH
  MODULE schema,
  material_scalars := (
    SELECT ScalarType
    FILTER NOT .abstract
       AND NOT EXISTS .enum_values
       AND NOT EXISTS (SELECT .ancestors FILTER NOT .abstract)
  )

	SELECT Type {
	  id,
	  name :=
	    array_join(array_agg([IS ObjectType].union_of.name), ' | ')
	    IF EXISTS [IS ObjectType].union_of
	    ELSE .name,
	  is_abstract := .abstract,

	  kind := 'object' IF Type IS ObjectType ELSE
	          'scalar' IF Type IS ScalarType ELSE
	          'array' IF Type IS Array ELSE
	          'tuple' IF Type IS Tuple ELSE
	          'multirange' IF Type IS MultiRange ELSE
	          'unknown',

	  [IS ScalarType].enum_values,
	  is_seq := 'std::sequence' in [IS ScalarType].ancestors.name,
	  # for sequence (abstract type that has non-abstract ancestor)
	  single material_id := (
	    SELECT x := Type[IS ScalarType].ancestors
	    FILTER x IN material_scalars
	    LIMIT 1
	  ).id,

	  [IS InheritingObject].bases: {
	    id
	  } ORDER BY @index ASC,

	  [IS ObjectType].union_of,
	  [IS ObjectType].intersection_of,
	  [IS ObjectType].pointers: {
	    card := ('One' IF .required ELSE 'AtMostOne') IF <str>.cardinality = 'One' ELSE ('AtLeastOne' IF .required ELSE 'Many'),
	    name,
	    target_id := .target.id,
	    kind := 'link' IF .__type__.name = 'schema::Link' ELSE 'property',
	    is_exclusive := exists (select .constraints filter .name = 'std::exclusive'),
	    is_computed := len(.computed_fields) != 0,
	    is_readonly := .readonly,
	    has_default := EXISTS .default or ('std::sequence' in .target[IS ScalarType].ancestors.name),
	    [IS Link].pointers: {
	      card := ('One' IF .required ELSE 'AtMostOne') IF <str>.cardinality = "One" ELSE ('AtLeastOne' IF .required ELSE 'Many'),
	      name := '@' ++ .name,
	      target_id := .target.id,
	      kind := 'link' IF .__type__.name = 'schema::Link' ELSE 'property',
	      is_computed := len(.computed_fields) != 0,
	      is_readonly := .readonly
	    } filter .name != '@source' and .name != '@target',
	  } FILTER @is_owned,
	  exclusives := assert_distinct((
	    [is schema::ObjectType].constraints
	    union
	    [is schema::ObjectType].pointers.constraints
	  ) {
	    target := (.subject[is schema::Property].name ?? .subject[is schema::Link].name ?? .subjectexpr)
	  } filter .name = 'std::exclusive'),
	  backlinks := (
	     SELECT DETACHED Link
	     FILTER .target = Type
	       AND NOT EXISTS .source[IS ObjectType].union_of
	    ) {
	    card := 'AtMostOne'
	      IF
	      EXISTS (select .constraints filter .name = 'std::exclusive')
	      ELSE
	      'Many',
	    name := '<' ++ .name ++ '[is ' ++ assert_exists(.source.name) ++ ']',
	    stub := .name,
	    target_id := .source.id,
	    kind := 'link',
	    is_exclusive := (EXISTS (select .constraints filter .name = 'std::exclusive')) AND <str>.cardinality = 'One',
	  },
	  backlink_stubs := array_agg((
	    WITH
	      stubs := DISTINCT (SELECT DETACHED Link FILTER .target = Type).name,
	      baseObjectId := (SELECT DETACHED ObjectType FILTER .name = 'std::BaseObject' LIMIT 1).id
	    FOR stub in { stubs }
	    UNION (
	      SELECT {
	        card := 'Many',
	        name := '<' ++ stub,
	        target_id := baseObjectId,
	        kind := 'link',
	        is_exclusive := false,
	      }
	    )
	  )),
	  array_element_id := [IS Array].element_type.id,

	  tuple_elements := (SELECT [IS Tuple].element_types {
	    target_id := .type.id,
	    name
	  } ORDER BY @index ASC),
		 multirange_element_id := [IS MultiRange].element_type.id,
	}
FILTER NOT .from_alias
ORDER BY .name;
"#;
