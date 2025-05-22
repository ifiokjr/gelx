pub mod example {
    use ::gelx::exports as __g;
    /// Execute the desired query.
    pub async fn query(
        client: &__g::gel_tokio::Client,
    ) -> ::core::result::Result<Vec<Output>, __g::gel_errors::Error> {
        client.query(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut __g::gel_tokio::Transaction,
    ) -> ::core::result::Result<Vec<Output>, __g::gel_errors::Error> {
        conn.query(QUERY, &()).await
    }
    pub type Input = ();
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputBasesSet {
        pub id: __g::uuid::Uuid,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputUnionOfSet {
        pub id: __g::uuid::Uuid,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputIntersectionOfSet {
        pub id: __g::uuid::Uuid,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputPointersSetPointersSet {
        pub card: Option<String>,
        pub name: String,
        pub target_id: Option<__g::uuid::Uuid>,
        pub kind: String,
        pub is_computed: Option<bool>,
        pub is_readonly: Option<bool>,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputPointersSet {
        pub card: Option<String>,
        pub name: String,
        pub target_id: Option<__g::uuid::Uuid>,
        pub kind: String,
        pub is_exclusive: bool,
        pub is_computed: Option<bool>,
        pub is_readonly: Option<bool>,
        pub has_default: bool,
        pub pointers: Vec<OutputPointersSetPointersSet>,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputExclusivesSet {
        pub target: Option<String>,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputBacklinksSet {
        pub card: String,
        pub name: String,
        pub stub: String,
        pub target_id: Option<__g::uuid::Uuid>,
        pub kind: String,
        pub is_exclusive: Option<bool>,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputBacklinkStubsArray {
        pub card: String,
        pub name: String,
        pub target_id: Option<__g::uuid::Uuid>,
        pub kind: String,
        pub is_exclusive: bool,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct OutputTupleElementsSet {
        pub target_id: __g::uuid::Uuid,
        pub name: Option<String>,
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct Output {
        pub id: __g::uuid::Uuid,
        pub name: String,
        pub is_abstract: Option<bool>,
        pub kind: String,
        pub enum_values: Option<Vec<String>>,
        pub is_seq: bool,
        pub material_id: Option<__g::uuid::Uuid>,
        pub bases: Vec<OutputBasesSet>,
        pub union_of: Vec<OutputUnionOfSet>,
        pub intersection_of: Vec<OutputIntersectionOfSet>,
        pub pointers: Vec<OutputPointersSet>,
        pub exclusives: Vec<OutputExclusivesSet>,
        pub backlinks: Vec<OutputBacklinksSet>,
        pub backlink_stubs: Vec<OutputBacklinkStubsArray>,
        pub array_element_id: Option<__g::uuid::Uuid>,
        pub tuple_elements: Vec<OutputTupleElementsSet>,
        pub multirange_element_id: Option<__g::uuid::Uuid>,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "WITH\n  MODULE schema,\n  material_scalars := (\n    SELECT ScalarType\n    FILTER NOT .abstract\n       AND NOT EXISTS .enum_values\n       AND NOT EXISTS (SELECT .ancestors FILTER NOT .abstract)\n  )\n\n\tSELECT Type {\n\t  id,\n\t  name :=\n\t    array_join(array_agg([IS ObjectType].union_of.name), ' | ')\n\t    IF EXISTS [IS ObjectType].union_of\n\t    ELSE .name,\n\t  is_abstract := .abstract,\n\n\t  kind := 'object' IF Type IS ObjectType ELSE\n\t          'scalar' IF Type IS ScalarType ELSE\n\t          'array' IF Type IS Array ELSE\n\t          'tuple' IF Type IS Tuple ELSE\n\t          'multirange' IF Type IS MultiRange ELSE\n\t          'unknown',\n\n\t  [IS ScalarType].enum_values,\n\t  is_seq := 'std::sequence' in [IS ScalarType].ancestors.name,\n\t  # for sequence (abstract type that has non-abstract ancestor)\n\t  single material_id := (\n\t    SELECT x := Type[IS ScalarType].ancestors\n\t    FILTER x IN material_scalars\n\t    LIMIT 1\n\t  ).id,\n\n\t  [IS InheritingObject].bases: {\n\t    id\n\t  } ORDER BY @index ASC,\n\n\t  [IS ObjectType].union_of,\n\t  [IS ObjectType].intersection_of,\n\t  [IS ObjectType].pointers: {\n\t    card := ('One' IF .required ELSE 'AtMostOne') IF <str>.cardinality = 'One' ELSE ('AtLeastOne' IF .required ELSE 'Many'),\n\t    name,\n\t    target_id := .target.id,\n\t    kind := 'link' IF .__type__.name = 'schema::Link' ELSE 'property',\n\t    is_exclusive := exists (select .constraints filter .name = 'std::exclusive'),\n\t    is_computed := len(.computed_fields) != 0,\n\t    is_readonly := .readonly,\n\t    has_default := EXISTS .default or ('std::sequence' in .target[IS ScalarType].ancestors.name),\n\t    [IS Link].pointers: {\n\t      card := ('One' IF .required ELSE 'AtMostOne') IF <str>.cardinality = \"One\" ELSE ('AtLeastOne' IF .required ELSE 'Many'),\n\t      name := '@' ++ .name,\n\t      target_id := .target.id,\n\t      kind := 'link' IF .__type__.name = 'schema::Link' ELSE 'property',\n\t      is_computed := len(.computed_fields) != 0,\n\t      is_readonly := .readonly\n\t    } filter .name != '@source' and .name != '@target',\n\t  } FILTER @is_owned,\n\t  exclusives := assert_distinct((\n\t    [is schema::ObjectType].constraints\n\t    union\n\t    [is schema::ObjectType].pointers.constraints\n\t  ) {\n\t    target := (.subject[is schema::Property].name ?? .subject[is schema::Link].name ?? .subjectexpr)\n\t  } filter .name = 'std::exclusive'),\n\t  backlinks := (\n\t     SELECT DETACHED Link\n\t     FILTER .target = Type\n\t       AND NOT EXISTS .source[IS ObjectType].union_of\n\t    ) {\n\t    card := 'AtMostOne'\n\t      IF\n\t      EXISTS (select .constraints filter .name = 'std::exclusive')\n\t      ELSE\n\t      'Many',\n\t    name := '<' ++ .name ++ '[is ' ++ assert_exists(.source.name) ++ ']',\n\t    stub := .name,\n\t    target_id := .source.id,\n\t    kind := 'link',\n\t    is_exclusive := (EXISTS (select .constraints filter .name = 'std::exclusive')) AND <str>.cardinality = 'One',\n\t  },\n\t  backlink_stubs := array_agg((\n\t    WITH\n\t      stubs := DISTINCT (SELECT DETACHED Link FILTER .target = Type).name,\n\t      baseObjectId := (SELECT DETACHED ObjectType FILTER .name = 'std::BaseObject' LIMIT 1).id\n\t    FOR stub in { stubs }\n\t    UNION (\n\t      SELECT {\n\t        card := 'Many',\n\t        name := '<' ++ stub,\n\t        target_id := baseObjectId,\n\t        kind := 'link',\n\t        is_exclusive := false,\n\t      }\n\t    )\n\t  )),\n\t  array_element_id := [IS Array].element_type.id,\n\n\t  tuple_elements := (SELECT [IS Tuple].element_types {\n\t    target_id := .type.id,\n\t    name\n\t  } ORDER BY @index ASC),\n\t\t multirange_element_id := [IS MultiRange].element_type.id,\n\t}\nORDER BY .name;\n";
}
fn main() {}
