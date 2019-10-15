
use std::borrow::{Borrow, Cow};
use std::collections::HashSet;
use syn;
use syn::Lit::Str;
use syn::MetaItem::{List, NameValue, Word};
use syn::NestedMetaItem::MetaItem;
use ::context::Context;

struct Attribute<'a, T> {
    context: &'a Context,
    name: &'static str,
    value: Option<T>
}

impl<'a, T> Attribute<'a, T> {

    fn new(context: &'a Context, name: &'static str) -> Self {
        Attribute {
            context: context,
            name: name,
            value: None
        }
    }

    fn set(&mut self, value: T) {
        if self.value.is_some() {
            self.context.error(&format!("Duplicate attribute provided: {}", self.name));
        }
        else {
            self.value = Some(value);
        }
    }

    fn into(mut self) -> Option<T> {
        self.value.take()
    }

}

struct BoolAttribute<'a> {
    inner: Attribute<'a, ()>
}

impl<'a> BoolAttribute<'a> {

    fn new(context: &'a Context, name: &'static str) -> Self {
        BoolAttribute {
            inner: Attribute::new(context, name)
        }
    }

    fn set(&mut self) {
        self.inner.set(());
    }

    fn into(self) -> bool {
        self.inner.into().is_some()
    }

}

pub struct Container<'a> {
    ast: &'a syn::DeriveInput,

    fields: Vec<Field<'a>>
}

impl<'a> Container<'a> {

    pub fn from_ast(ast: &'a syn::DeriveInput) -> Result<Self, String> {
        let ctx = Context::new();

        let variant = match ast.body {
            syn::Body::Struct(ref variant) => variant,
            _ => return Err("#[derive(FormatArgs)] is not implemented for enums".to_string())
        };

        let fields = {
            let mut fields = Vec::new();
            let mut field_names = HashSet::new();
            for (tuple_index, field) in variant.fields().iter().enumerate() {
                if let Some(field) = Field::from_ast(&ctx, field, fields.len(), tuple_index) {
                    for alias in field.aliases() {
                        if !field_names.insert(*alias) {
                            ctx.error(&format!("Duplicate field alias: {}", alias));
                        }
                    }
                    fields.push(field);
                }
            }

            fields
        };

        ctx.check().map(|_| {
            Container {
                ast: ast,

                fields: fields
            }
        })
    }

    pub fn ident(&self) -> &'a syn::Ident {
        &self.ast.ident
    }

    pub fn generics(&self) -> &'a syn::Generics {
        &self.ast.generics
    }

    pub fn fields(&self) -> &[Field<'a>] {
        &self.fields
    }

}

pub struct Field<'a> {
    field_index: usize,

    ident: Cow<'a, syn::Ident>,
    aliases: Vec<&'a str>,

    ty: &'a syn::Ty
}

impl<'a> Field<'a> {

    pub fn from_ast(ctx: &Context, ast: &'a syn::Field, field_index: usize, tuple_index: usize) -> Option<Self> {
        let ident = match ast.ident {
            Some(ref ident) => Cow::Borrowed(ident),
            None => Cow::Owned(syn::Ident::from(tuple_index))
        };

        let mut aliases = Attribute::new(ctx, "aliases");
        let mut ignored = BoolAttribute::new(ctx, "ignore");
        let mut name = Attribute::new(ctx, "rename");

        for attributes in ast.attrs.iter().filter_map(filter_format_attributes) {
            for attribute in attributes {
                match *attribute {
                    MetaItem(NameValue(ref ident, Str(ref value, _))) if ident == "aliases" => {
                        aliases.set(value.split(",").collect());
                    }
                    MetaItem(NameValue(ref ident, Str(ref value, _))) if ident == "rename" => {
                        name.set(value.as_ref());
                    }
                    MetaItem(Word(ref ident)) if ident == "ignore" => {
                        ignored.set();
                    }
                    _ => ctx.error(&format!("Unrecognized attribute: {:?}", attribute))
                }
            }
        }

        let mut aliases = aliases.into().unwrap_or_else(Vec::new);
        if let Some(name) = name.into().or(ast.ident.as_ref().map(|ident| ident.as_ref())) {
            aliases.push(name);
        }

        if !ignored.into() {
            Some(Field {
                field_index: field_index,

                ident: ident,
                aliases: aliases,

                ty: &ast.ty
            })
        }
        else {
            None
        }
    }

    pub fn index(&self) -> usize {
        self.field_index
    }

    pub fn ident(&self) -> &syn::Ident {
        self.ident.borrow()
    }

    pub fn aliases(&self) -> &[&'a str] {
        &self.aliases
    }

    pub fn ty(&self) -> &'a syn::Ty {
        self.ty
    }

}

fn filter_format_attributes(attr: &syn::Attribute) -> Option<&Vec<syn::NestedMetaItem>> {
    match attr.value {
        List(ref name, ref items) if name == "format_args" => {
            Some(items)
        }
        _ => None
    }
}