use hashbrown::HashSet;
use iref::Iri;
use json_syntax::Parse;
use linked_data::{FromLinkedDataError, LinkedDataDeserialize};
use rdf_types::{
	dataset::{PatternMatchingDataset, TraversableDataset},
	interpretation::{
		ReverseIdInterpretation, ReverseIriInterpretation, ReverseTermInterpretation,
	},
	vocabulary::{BlankIdVocabulary, IriVocabulary},
	LiteralTypeRef, Quad, Term, Vocabulary,
};
use static_iref::iri;
use std::{
	collections::{BTreeMap, BTreeSet},
	hash::Hash,
	str::FromStr,
};

use crate::{
	object::{List, Literal},
	rdf::{
		RDF_FIRST, RDF_JSON, RDF_NIL, RDF_REST, RDF_TYPE, XSD_BOOLEAN, XSD_DOUBLE, XSD_INTEGER,
		XSD_STRING,
	},
	ExpandedDocument, Id, Indexed, IndexedObject, LangString, Node, Object, ValidId, Value,
};

struct SerDataset<R> {
	named_graphs: BTreeMap<R, SerGraph<R>>,
	default_graph: SerGraph<R>,
}

impl<R> SerDataset<R> {
	fn new() -> Self {
		Self {
			named_graphs: BTreeMap::new(),
			default_graph: SerGraph::new(),
		}
	}
}

impl<R: Ord> SerDataset<R> {
	fn graph_mut(&mut self, label: Option<R>) -> &mut SerGraph<R>
	where
		R: Ord,
	{
		match label {
			Some(g) => self.named_graphs.entry(g).or_insert_with(SerGraph::new),
			None => &mut self.default_graph,
		}
	}

	fn fold_into_default_graph(mut self) -> SerGraph<R> {
		for (id, graph) in self.named_graphs {
			self.default_graph.resource_mut(id).graph = Some(graph);
		}

		self.default_graph
	}
}

struct SerGraph<R> {
	resources: BTreeMap<R, SerResource<R>>,
}

struct SerList<R> {
	first: HashSet<R>,
	rest: HashSet<R>,
	reverse_rest: HashSet<R>,
	values: Option<Vec<R>>,
}

impl<R> Default for SerList<R> {
	fn default() -> Self {
		Self {
			first: HashSet::new(),
			rest: HashSet::new(),
			reverse_rest: HashSet::new(),
			values: None,
		}
	}
}

impl<R> SerList<R> {
	fn is_well_formed(&self) -> bool {
		self.first.len() == 1 && self.rest.len() == 1
	}

	fn is_empty(&self) -> bool {
		self.first.is_empty() && self.rest.is_empty()
	}
}

struct SerResource<R> {
	types: BTreeSet<RdfType<R>>,
	properties: BTreeMap<R, BTreeSet<R>>,
	graph: Option<SerGraph<R>>,
	list: SerList<R>,
	references: usize,
}

impl<R> Default for SerResource<R> {
	fn default() -> Self {
		Self {
			types: BTreeSet::new(),
			properties: BTreeMap::new(),
			graph: None,
			list: SerList::default(),
			references: 0,
		}
	}
}

impl<R> SerResource<R> {
	fn is_empty(&self) -> bool {
		self.types.is_empty()
			&& self.properties.is_empty()
			&& self.graph.is_none()
			&& self.list.is_empty()
	}

	fn is_list_node(&self) -> bool {
		self.types.iter().all(|ty| ty.is_list())
			&& self.properties.is_empty()
			&& self.graph.is_none()
			&& self.list.is_well_formed()
	}

	fn insert(&mut self, prop: R, object: R)
	where
		R: Ord,
	{
		self.properties.entry(prop).or_default().insert(object);
	}
}

impl<R> SerGraph<R> {
	fn new() -> Self {
		Self {
			resources: BTreeMap::new(),
		}
	}

	fn get(&self, id: &R) -> Option<&SerResource<R>>
	where
		R: Ord,
	{
		self.resources.get(id)
	}

	fn resource_mut(&mut self, id: R) -> &mut SerResource<R>
	where
		R: Ord,
	{
		self.resources.entry(id).or_default()
	}
}

enum RdfProperty {
	Type,
	First,
	Rest,
}

fn rdf_property<V: IriVocabulary, I: ReverseIriInterpretation<Iri = V::Iri>>(
	vocabulary: &V,
	interpretation: &I,
	id: &I::Resource,
) -> Option<RdfProperty> {
	for i in interpretation.iris_of(id) {
		let iri = vocabulary.iri(i).unwrap();
		if iri == RDF_TYPE {
			return Some(RdfProperty::Type);
		} else if iri == RDF_FIRST {
			return Some(RdfProperty::First);
		} else if iri == RDF_REST {
			return Some(RdfProperty::Rest);
		}
	}

	None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RdfType<R> {
	List,
	Other(R),
}

impl<R> RdfType<R> {
	fn is_list(&self) -> bool {
		matches!(self, Self::List)
	}
}

const RDF_LIST: &Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#List");

fn rdf_type<'a, V: IriVocabulary, I: ReverseIriInterpretation<Iri = V::Iri>>(
	vocabulary: &V,
	interpretation: &I,
	id: &'a I::Resource,
) -> RdfType<&'a I::Resource> {
	for i in interpretation.iris_of(id) {
		let iri = vocabulary.iri(i).unwrap();
		if iri == RDF_LIST {
			return RdfType::List;
		}
	}

	RdfType::Other(id)
}

fn is_anonymous<I: ReverseTermInterpretation>(interpretation: &I, id: &I::Resource) -> bool {
	interpretation.iris_of(id).next().is_none() && interpretation.literals_of(id).next().is_none()
}

#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
	#[error("invalid JSON")]
	InvalidJson(linked_data::ContextIris, json_syntax::parse::Error),

	#[error("invalid boolean value")]
	InvalidBoolean(linked_data::ContextIris, String),

	#[error("invalid number value")]
	Number(linked_data::ContextIris, String),
}

#[derive(Clone, Copy)]
pub struct RdfTerms<R> {
	list: Option<R>,
	first: Option<R>,
	rest: Option<R>,
}

impl<I, B> ExpandedDocument<I, B> {
	pub fn from_interpreted_quads_in<'a, V, T>(
		vocabulary: &V,
		interpretation: &T,
		quads: impl IntoIterator<
			Item = Quad<&'a T::Resource, &'a T::Resource, &'a T::Resource, &'a T::Resource>,
		>,
		context: linked_data::Context<T>,
	) -> Result<Self, SerializationError>
	where
		V: Vocabulary<Iri = I, BlankId = B>,
		T: ReverseTermInterpretation<Iri = I, BlankId = B, Literal = V::Literal>,
		T::Resource: 'a + Ord + Hash,
		I: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		let mut node_map: SerDataset<&'a T::Resource> = SerDataset::new();

		let mut nil = None;
		let mut rdf_terms = RdfTerms {
			list: None,
			first: None,
			rest: None,
		};

		for quad in quads {
			let graph = node_map.graph_mut(quad.3);
			let subject = graph.resource_mut(quad.0);

			match rdf_property(vocabulary, interpretation, quad.1) {
				Some(RdfProperty::Type) => {
					rdf_terms.first = Some(quad.1);
					let ty = rdf_type(vocabulary, interpretation, quad.2);

					if ty.is_list() {
						rdf_terms.list = Some(quad.2);
					}

					subject.types.insert(ty);
				}
				Some(RdfProperty::First) => {
					rdf_terms.first = Some(quad.1);
					subject.list.first.insert(quad.2);
				}
				Some(RdfProperty::Rest) => {
					rdf_terms.rest = Some(quad.1);
					if nil.is_none() {
						for i in interpretation.iris_of(quad.2) {
							let iri = vocabulary.iri(i).unwrap();
							if iri == RDF_NIL {
								nil = Some(quad.2);
							}
						}
					}

					subject.list.rest.insert(quad.2);
					graph.resource_mut(quad.2).list.reverse_rest.insert(quad.1);
				}
				None => {
					subject.insert(quad.1, quad.2);
				}
			}

			let object = graph.resource_mut(quad.2);
			if quad.1 == quad.2 {
				object.references = usize::MAX;
			} else {
				let r = object.references;
				object.references = r.saturating_add(1)
			}
		}

		let mut graph = node_map.fold_into_default_graph();

		let mut lists = Vec::new();
		if let Some(nil_id) = nil {
			if let Some(nil) = graph.get(&nil_id) {
				for &node_id in &nil.list.reverse_rest {
					let mut head_id = node_id;
					if is_anonymous(interpretation, head_id) {
						if let Some(mut head) = graph.get(&head_id) {
							if head.references == 1 && head.is_list_node() {
								let mut values = Vec::new();

								loop {
									let first = head.list.first.iter().next().copied().unwrap();
									let parent_id =
										head.list.reverse_rest.iter().next().copied().unwrap();
									values.push(first);

									if is_anonymous(interpretation, parent_id) {
										if let Some(parent) = graph.get(&parent_id) {
											if parent.references == 1 && parent.is_list_node() {
												head_id = parent_id;
												head = parent;
												continue;
											}
										}
									}

									break;
								}

								values.reverse();
								lists.push((head_id, values))
							}
						}
					}
				}
			}
		}

		for (id, values) in lists {
			graph.resource_mut(id).list.values = Some(values)
		}

		let mut result = ExpandedDocument::new();
		for (id, resource) in &graph.resources {
			if resource.references != 1 && !resource.is_empty() {
				result.insert(render_object(
					vocabulary,
					interpretation,
					rdf_terms,
					&graph,
					id,
					resource,
					context,
				)?);
			}
		}

		Ok(result)
	}

	pub fn from_interpreted_quads<'a, V, T>(
		vocabulary: &V,
		interpretation: &T,
		quads: impl IntoIterator<
			Item = Quad<&'a T::Resource, &'a T::Resource, &'a T::Resource, &'a T::Resource>,
		>,
	) -> Result<Self, SerializationError>
	where
		V: Vocabulary<Iri = I, BlankId = B>,
		T: ReverseTermInterpretation<Iri = I, BlankId = B, Literal = V::Literal>,
		T::Resource: 'a + Ord + Hash,
		I: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		Self::from_interpreted_quads_in(
			vocabulary,
			interpretation,
			quads,
			linked_data::Context::default(),
		)
	}
}

fn render_object<V, I>(
	vocabulary: &V,
	interpretation: &I,
	rdf_terms: RdfTerms<&I::Resource>,
	graph: &SerGraph<&I::Resource>,
	id: &I::Resource,
	resource: &SerResource<&I::Resource>,
	context: linked_data::Context<I>,
) -> Result<IndexedObject<V::Iri, V::BlankId>, SerializationError>
where
	V: Vocabulary,
	I: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I::Resource: Ord,
{
	let context = context.with_subject(id);
	if resource.is_empty() {
		render_reference(vocabulary, interpretation, id, context)
	} else {
		match &resource.list.values {
			Some(values) => {
				let mut objects = Vec::with_capacity(values.len());

				for value in values {
					objects.push(render_object_or_reference(
						vocabulary,
						interpretation,
						rdf_terms,
						graph,
						value,
						context,
					)?);
				}

				Ok(Indexed::none(Object::List(List::new(objects))))
			}
			None => {
				let mut node: Node<V::Iri, V::BlankId> = Node::new();

				if let Some(id) = id_of(interpretation, id) {
					node.id = Some(id)
				}

				let mut types = Vec::with_capacity(resource.types.len());
				for ty in &resource.types {
					let ty_resource = match ty {
						RdfType::List => rdf_terms.list.unwrap(),
						RdfType::Other(o) => o,
					};

					if let Some(ty_id) = id_of(interpretation, ty_resource) {
						types.push(ty_id)
					}
				}

				if !types.is_empty() {
					node.types = Some(types);
				}

				if let Some(graph) = &resource.graph {
					let mut value = crate::object::Graph::new();

					for (id, resource) in &graph.resources {
						if resource.references != 1 && !resource.is_empty() {
							value.insert(render_object(
								vocabulary,
								interpretation,
								rdf_terms,
								graph,
								id,
								resource,
								context,
							)?);
						}
					}

					node.graph = Some(value)
				}

				for (prop, objects) in &resource.properties {
					insert_property(
						vocabulary,
						interpretation,
						rdf_terms,
						graph,
						&mut node,
						prop,
						objects.iter().copied(),
						context,
					)?;
				}

				if !resource.list.first.is_empty() {
					let rdf_first_id = rdf_terms.first.unwrap();
					insert_property(
						vocabulary,
						interpretation,
						rdf_terms,
						graph,
						&mut node,
						rdf_first_id,
						resource.list.first.iter().copied(),
						context,
					)?;
				}

				if !resource.list.rest.is_empty() {
					let rdf_rest_id = rdf_terms.rest.unwrap();
					insert_property(
						vocabulary,
						interpretation,
						rdf_terms,
						graph,
						&mut node,
						rdf_rest_id,
						resource.list.rest.iter().copied(),
						context,
					)?;
				}

				Ok(Indexed::none(Object::node(node)))
			}
		}
	}
}

#[allow(clippy::too_many_arguments)]
fn insert_property<'a, V, I, O>(
	vocabulary: &V,
	interpretation: &I,
	rdf_terms: RdfTerms<&'a I::Resource>,
	graph: &SerGraph<&'a I::Resource>,
	node: &mut Node<V::Iri, V::BlankId>,
	prop: &I::Resource,
	values: O,
	context: linked_data::Context<I>,
) -> Result<(), SerializationError>
where
	V: Vocabulary,
	I: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I::Resource: 'a + Ord,
	O: IntoIterator<Item = &'a I::Resource>,
	O::IntoIter: ExactSizeIterator,
{
	let context = context.with_predicate(prop);
	match id_of(interpretation, prop) {
		Some(prop) => {
			let mut values = values.into_iter();

			while values.len() > 1 {
				let value = values.next().unwrap();
				let v = render_object_or_reference(
					vocabulary,
					interpretation,
					rdf_terms,
					graph,
					value,
					context,
				)?;
				node.insert(prop.clone(), v);
			}

			if let Some(value) = values.next() {
				let v = render_object_or_reference(
					vocabulary,
					interpretation,
					rdf_terms,
					graph,
					value,
					context,
				)?;
				node.insert(prop, v);
			}

			Ok(())
		}
		None => Ok(()),
	}
}

fn render_object_or_reference<V, I>(
	vocabulary: &V,
	interpretation: &I,
	rdf_terms: RdfTerms<&I::Resource>,
	graph: &SerGraph<&I::Resource>,
	id: &I::Resource,
	context: linked_data::Context<I>,
) -> Result<IndexedObject<V::Iri, V::BlankId>, SerializationError>
where
	V: Vocabulary,
	I: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I::Resource: Ord,
{
	match graph.get(&id) {
		Some(resource) => {
			if resource.references == 1 && !resource.is_empty() {
				render_object(
					vocabulary,
					interpretation,
					rdf_terms,
					graph,
					id,
					resource,
					context,
				)
			} else {
				render_reference(vocabulary, interpretation, id, context)
			}
		}
		None => render_reference(vocabulary, interpretation, id, context),
	}
}

fn render_reference<V, I>(
	vocabulary: &V,
	interpretation: &I,
	id: &I::Resource,
	context: linked_data::Context<I>,
) -> Result<IndexedObject<V::Iri, V::BlankId>, SerializationError>
where
	V: Vocabulary,
	I: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	V::Iri: Clone,
	V::BlankId: Clone,
	I::Resource: Ord,
{
	match term_of(vocabulary, interpretation, id, context)? {
		Some(Term::Id(id)) => Ok(Indexed::none(Object::node(Node::with_id(id)))),
		Some(Term::Literal(value)) => Ok(Indexed::none(Object::Value(value))),
		None => Ok(Indexed::none(Object::node(Node::new()))),
	}
}

fn id_of<T>(interpretation: &T, resource: &T::Resource) -> Option<Id<T::Iri, T::BlankId>>
where
	T: ReverseIdInterpretation,
	T::Iri: Clone,
	T::BlankId: Clone,
{
	interpretation
		.iris_of(resource)
		.next()
		.map(|i| Id::Valid(ValidId::Iri(i.clone())))
		.or_else(|| {
			interpretation
				.blank_ids_of(resource)
				.next()
				.map(|b| Id::Valid(ValidId::Blank(b.clone())))
		})
}

type ResourceTerm<V> = Term<
	Id<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId>,
	Value<<V as IriVocabulary>::Iri>,
>;

fn term_of<V, T>(
	vocabulary: &V,
	interpretation: &T,
	resource: &T::Resource,
	context: linked_data::Context<T>,
) -> Result<Option<ResourceTerm<V>>, SerializationError>
where
	V: Vocabulary,
	T: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	V::Iri: Clone,
	V::BlankId: Clone,
{
	match id_of(interpretation, resource) {
		Some(id) => Ok(Some(Term::Id(id))),
		None => match interpretation.literals_of(resource).next() {
			Some(l) => {
				let l = vocabulary.literal(l).unwrap();
				let value = match l.type_ {
					LiteralTypeRef::Any(i) => {
						let ty = vocabulary.iri(i).unwrap();
						if ty == RDF_JSON {
							let (json, _) =
								json_syntax::Value::parse_str(l.value).map_err(|e| {
									SerializationError::InvalidJson(
										context.into_iris(vocabulary, interpretation),
										e,
									)
								})?;
							Value::Json(json)
						} else if ty == XSD_BOOLEAN {
							let b = match l.as_ref() {
								"true" | "1" => true,
								"false" | "0" => false,
								other => {
									return Err(SerializationError::InvalidBoolean(
										context.into_iris(vocabulary, interpretation),
										other.to_owned(),
									))
								}
							};

							Value::Literal(Literal::Boolean(b), Some(i.clone()))
						} else if ty == XSD_INTEGER || ty == XSD_DOUBLE {
							let n = json_syntax::NumberBuf::from_str(l.as_str()).map_err(|_| {
								SerializationError::Number(
									context.into_iris(vocabulary, interpretation),
									l.as_ref().to_owned(),
								)
							})?;
							Value::Literal(Literal::Number(n), Some(i.clone()))
						} else if ty == XSD_STRING {
							Value::Literal(Literal::String(l.as_ref().into()), None)
						} else {
							Value::Literal(Literal::String(l.as_ref().into()), Some(i.clone()))
						}
					}
					LiteralTypeRef::LangString(tag) => Value::LangString(
						LangString::new(l.value.into(), Some(tag.to_owned().into()), None).unwrap(),
					),
				};

				Ok(Some(Term::Literal(value)))
			}
			None => Ok(None),
		},
	}
}

impl<V, I> LinkedDataDeserialize<V, I> for ExpandedDocument<V::Iri, V::BlankId>
where
	V: Vocabulary,
	I: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Ord + Hash,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
{
	fn deserialize_dataset_in(
		vocabulary: &V,
		interpretation: &I,
		dataset: &(impl TraversableDataset<Resource = I::Resource> + PatternMatchingDataset),
		context: linked_data::Context<I>,
	) -> Result<Self, FromLinkedDataError> {
		Self::from_interpreted_quads(vocabulary, interpretation, dataset.quads()).map_err(|_| {
			FromLinkedDataError::InvalidLiteral(context.into_iris(vocabulary, interpretation))
		})
	}
}
