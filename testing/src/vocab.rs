use iref_enum::IriEnum;

pub type IriIndex = json_ld::namespace::IriIndex<Vocab>;
pub type BlankIdIndex = json_ld::namespace::Index;

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vocab {
	Rdf(Rdf),
	Rdfs(Rdfs),
	Xsd(Xsd),
	Manifest(Manifest),
	Test(Test),
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("rdf" = "http://www.w3.org/1999/02/22-rdf-syntax-ns#")]
pub enum Rdf {
	#[iri("rdf:type")]
	Type,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
pub enum Rdfs {
	#[iri("rdfs:comment")]
	Comment,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("xsd" = "http://www.w3.org/2001/XMLSchema#")]
pub enum Xsd {
	#[iri("xsd:boolean")]
	Boolean,

	#[iri("xsd:string")]
	String,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
pub enum Manifest {
	#[iri("manifest:name")]
	Name,
	#[iri("manifest:entries")]
	Entries,
	#[iri("manifest:action")]
	Action,
	#[iri("manifest:result")]
	Result,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
pub enum Test {
	#[iri("test:PositiveEvaluationTest")]
	PositiveEvalTest,
	#[iri("test:NegativeEvaluationTest")]
	NegativeEvalTest,
	#[iri("test:context")]
	Context,
	#[iri("test:option")]
	Option,
	#[iri("test:base")]
	Base,
	#[iri("test:compactArrays")]
	CompactArrays,
	#[iri("test:processingMode")]
	ProcessingMode,
	#[iri("test:specVersion")]
	SpecVersion,
}
