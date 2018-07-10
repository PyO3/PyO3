// Copyright (c) 2017-present PyO3 Project and Contributors

use method::{FnArg, FnSpec, FnType};
use proc_macro2::{Span, TokenStream};
use py_method::{impl_py_getter_def, impl_py_setter_def, impl_wrap_getter, impl_wrap_setter};
use std::collections::HashMap;
use syn;
use utils;

pub fn build_py_class(class: &mut syn::ItemStruct, attr: &Vec<syn::Expr>) -> TokenStream {
    let (params, flags, base) = parse_attribute(attr);
    let doc = utils::get_doc(&class.attrs, true);
    let mut token: Option<syn::Ident> = None;
    let mut descriptors = Vec::new();

    if let syn::Fields::Named(ref mut fields) = class.fields {
        for field in fields.named.iter_mut() {
            if is_python_token(field) {
                if token.is_none() {
                    token = field.ident.clone();
                } else {
                    panic!("You can only have one PyToken per class");
                }
            } else {
                let field_descs = parse_descriptors(field);
                if !field_descs.is_empty() {
                    descriptors.push((field.clone(), field_descs));
                }
            }
        }
    } else {
        panic!("#[class] can only be used with C-style structs")
    }

    impl_class(&class, &base, token, doc, params, flags, descriptors)
}

fn parse_descriptors(item: &mut syn::Field) -> Vec<FnType> {
    let mut descs = Vec::new();
    let mut new_attrs = Vec::new();
    for attr in item.attrs.iter() {
        if let Some(syn::Meta::List(ref list)) = attr.interpret_meta() {
            match list.ident.to_string().as_str() {
                "prop" => {
                    for meta in list.nested.iter() {
                        if let &syn::NestedMeta::Meta(ref metaitem) = meta {
                            match metaitem.name().to_string().as_str() {
                                "get" => {
                                    descs.push(FnType::Getter(None));
                                }
                                "set" => {
                                    descs.push(FnType::Setter(None));
                                }
                                x => {
                                    panic!(r#"Only "get" and "set" supported are, not "{}""#, x);
                                }
                            }
                        }
                    }
                }
                _ => new_attrs.push(attr.clone()),
            }
        } else {
            new_attrs.push(attr.clone());
        }
    }
    item.attrs.clear();
    item.attrs.extend(new_attrs);
    descs
}

fn impl_class(
    class: &syn::ItemStruct,
    base: &syn::TypePath,
    token: Option<syn::Ident>,
    doc: syn::Lit,
    params: HashMap<&'static str, syn::Expr>,
    flags: Vec<syn::Expr>,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
) -> TokenStream {
    let cls = &class.ident;
    let generics = &class.generics;

    let cls_name = match params.get("name") {
        Some(name) => quote! { #name }.to_string(),
        None => quote! { #cls }.to_string(),
    };

    let extra = if let Some(token) = token {
        Some(quote! {
            impl #generics ::pyo3::PyObjectWithToken for #cls #generics {
                #[inline(always)]
                fn py<'p>(&'p self) -> ::pyo3::Python<'p> {
                    self.#token.py()
                }
            }
            impl #generics ::pyo3::ToPyObject for #cls #generics {
                #[inline]
                fn to_object<'p>(&self, py: ::pyo3::Python<'p>) -> ::pyo3::PyObject {
                    unsafe { ::pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
                }
            }
            impl #generics ::pyo3::ToBorrowedObject for #cls #generics {
                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: ::pyo3::Python, f: F) -> R
                    where F: FnOnce(*mut ::pyo3::ffi::PyObject) -> R
                {
                    f(self.as_ptr())
                }
            }
            impl<'a> #generics ::pyo3::ToPyObject for &'a mut #cls #generics {
                #[inline]
                fn to_object<'p>(&self, py: ::pyo3::Python<'p>) -> ::pyo3::PyObject {
                    unsafe { ::pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
                }
            }
            impl<'a> #generics ::pyo3::ToBorrowedObject for &'a mut #cls #generics {
                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: ::pyo3::Python, f: F) -> R
                    where F: FnOnce(*mut ::pyo3::ffi::PyObject) -> R
                {
                    f(self.as_ptr())
                }
            }
            impl<'a> #generics std::convert::From<&'a mut #cls> for &'a #cls #generics
            {
                fn from(ob: &'a mut #cls) -> Self {
                    unsafe{std::mem::transmute(ob)}
                }
            }
            impl #generics ::pyo3::ToPyPointer for #cls #generics {
                #[inline]
                fn as_ptr(&self) -> *mut ::pyo3::ffi::PyObject {
                    unsafe {
                        {self as *const _ as *mut u8}
                        .offset(-<#cls as ::pyo3::typeob::PyTypeInfo>::OFFSET) as *mut ::pyo3::ffi::PyObject
                    }
                }
            }
            impl #generics std::fmt::Debug for #cls #generics {
                fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    use pyo3::ObjectProtocol;
                    let s = try!(self.repr().map_err(|_| std::fmt::Error));
                    f.write_str(&s.to_string_lossy())
                }
            }
            impl #generics std::fmt::Display for #cls #generics {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    use pyo3::ObjectProtocol;
                    let s = try!(self.str().map_err(|_| std::fmt::Error));
                    f.write_str(&s.to_string_lossy())
                }
            }
        })
    } else {
        None
    };

    let extra = {
        if let Some(freelist) = params.get("freelist") {
            Some(quote! {
                impl #generics ::pyo3::freelist::PyObjectWithFreeList for #cls #generics {
                    #[inline]
                    fn get_free_list() -> &'static mut ::pyo3::freelist::FreeList<*mut ::pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut ::pyo3::freelist::FreeList<*mut ::pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = Box::into_raw(Box::new(
                                    ::pyo3::freelist::FreeList::with_capacity(#freelist)));

                                <#cls as ::pyo3::typeob::PyTypeObject>::init_type();
                            }
                            &mut *FREELIST
                        }
                    }
                }

                #extra
            })
        } else {
            extra
        }
    };

    let extra = if !descriptors.is_empty() {
        let ty = syn::parse_str(&cls.to_string()).expect("no name");
        let desc_impls = impl_descriptors(&ty, &generics, descriptors);
        Some(quote! {
            #desc_impls
            #extra
        })
    } else {
        extra
    };

    // insert space for weak ref
    let mut has_weakref = false;
    let mut has_dict = false;
    for f in flags.iter() {
        if let syn::Expr::Path(ref epath) = f {
            if epath.path == parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_WEAKREF} {
                has_weakref = true;
            } else if epath.path == parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_DICT} {
                has_dict = true;
            }
        }
    }
    let weakref = if has_weakref {
        quote! {std::mem::size_of::<*const ::pyo3::ffi::PyObject>()}
    } else {
        quote! {0}
    };
    let dict = if has_dict {
        quote! {std::mem::size_of::<*const ::pyo3::ffi::PyObject>()}
    } else {
        quote! {0}
    };

    quote! {
        impl #generics ::pyo3::typeob::PyTypeInfo for #cls #generics {
            type Type = #cls #generics;
            type BaseType = #base;

            const NAME: &'static str = #cls_name;
            const DESCRIPTION: &'static str = #doc;
            const FLAGS: usize = #(#flags)|*;

            const SIZE: usize = {
                Self::OFFSET as usize +
                std::mem::size_of::<#cls>() + #weakref + #dict
            };
            const OFFSET: isize = {
                // round base_size up to next multiple of align
                (
                    (<#base as ::pyo3::typeob::PyTypeInfo>::SIZE +
                     std::mem::align_of::<#cls>() - 1)  /
                        std::mem::align_of::<#cls>() * std::mem::align_of::<#cls>()
                ) as isize
            };

            #[inline]
            unsafe fn type_object() -> &'static mut ::pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: ::pyo3::ffi::PyTypeObject = ::pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }
        }

        impl #generics ::pyo3::typeob::PyTypeObject for #cls #generics {
            #[inline(always)]
            fn init_type() {
                static START: std::sync::Once = std::sync::ONCE_INIT;
                START.call_once(|| {
                    let ty = unsafe{<#cls as ::pyo3::typeob::PyTypeInfo>::type_object()};

                    if (ty.tp_flags & ::pyo3::ffi::Py_TPFLAGS_READY) == 0 {
                        let gil = ::pyo3::Python::acquire_gil();
                        let py = gil.python();

                        // automatically initialize the class on-demand
                        ::pyo3::typeob::initialize_type::<#cls #generics>(py, None)
                            .map_err(|e| e.print(py))
                            .expect(format!("An error occurred while initializing class {}",
                                            <#cls as ::pyo3::typeob::PyTypeInfo>::NAME).as_ref());
                    }
                });
            }
        }

        #extra
    }
}

fn impl_descriptors(cls: &syn::Type, generics: &syn::Generics, descriptors: Vec<(syn::Field, Vec<FnType>)>) -> TokenStream {
    let methods: Vec<TokenStream> = descriptors
        .iter()
        .flat_map(|&(ref field, ref fns)| {
            fns.iter()
                .map(|desc| {
                    let name = field.ident.clone().unwrap();
                    let field_ty = &field.ty;
                    match *desc {
                        FnType::Getter(_) => {
                            quote! {
                                impl #generics #cls #generics {
                                    fn #name(&self) -> ::pyo3::PyResult<#field_ty> {
                                        Ok(self.#name.clone())
                                    }
                                }
                            }
                        }
                        FnType::Setter(_) => {
                            let setter_name =
                                syn::Ident::new(&format!("set_{}", name), Span::call_site());
                            quote! {
                                impl #generics #cls #generics {
                                    fn #setter_name(&mut self, value: #field_ty) -> ::pyo3::PyResult<()> {
                                        self.#name = value;
                                        Ok(())
                                    }
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<TokenStream>>()
        })
        .collect();

    let py_methods: Vec<TokenStream> = descriptors
        .iter()
        .flat_map(|&(ref field, ref fns)| {
            fns.iter()
                .map(|desc| {
                    let name = field.ident.clone().unwrap();

                    // FIXME better doc?
                    let doc: syn::Lit = syn::parse_str(&format!("\"{}\"", name)).unwrap();

                    let field_ty = &field.ty;
                    match *desc {
                        FnType::Getter(ref getter) => {
                            impl_py_getter_def(&name, doc, getter, &impl_wrap_getter(&cls, &name))
                        }
                        FnType::Setter(ref setter) => {
                            let setter_name =
                                syn::Ident::new(&format!("set_{}", name), Span::call_site());
                            let spec = FnSpec {
                                tp: FnType::Setter(None),
                                attrs: Vec::new(),
                                args: vec![FnArg {
                                    name: &name,
                                    mutability: &None,
                                    by_ref: &None,
                                    ty: field_ty,
                                    optional: None,
                                    py: true,
                                    reference: false,
                                }],
                                output: parse_quote!(PyResult<()>),
                            };
                            impl_py_setter_def(
                                &name,
                                doc,
                                setter,
                                &impl_wrap_setter(&cls, &setter_name, &spec),
                            )
                        }
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<TokenStream>>()
        })
        .collect();

    quote! {
        #(#methods)*

        impl #generics ::pyo3::class::methods::PyPropMethodsProtocolImpl for #cls #generics {
            fn py_methods() -> &'static [::pyo3::class::PyMethodDefType] {
                static METHODS: &'static [::pyo3::class::PyMethodDefType] = &[
                    #(#py_methods),*
                ];
                METHODS
            }
        }
    }
}

fn is_python_token(field: &syn::Field) -> bool {
    match field.ty {
        syn::Type::Path(ref typath) => {
            if let Some(segment) = typath.path.segments.last() {
                return segment.value().ident.to_string() == "PyToken";
            }
        }
        _ => (),
    }
    return false;
}

fn parse_attribute(
    args: &Vec<syn::Expr>,
) -> (
    HashMap<&'static str, syn::Expr>,
    Vec<syn::Expr>,
    syn::TypePath,
) {
    let mut params = HashMap::new();
    let mut flags = vec![syn::Expr::Lit(parse_quote! {0})];
    let mut base: syn::TypePath = parse_quote! {::pyo3::PyObjectRef};

    for expr in args.iter() {
        match expr {
            // Match a single flag
            syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => match exp
                .path
                .segments
                .first()
                .unwrap()
                .value()
                .ident
                .to_string()
                .as_str()
            {
                "gc" => {
                    flags.push(syn::Expr::Path(
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_GC},
                    ));
                }
                "weakref" => {
                    flags.push(syn::Expr::Path(
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_WEAKREF},
                    ));
                }
                "subclass" => {
                    flags.push(syn::Expr::Path(
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_BASETYPE},
                    ));
                }
                "dict" => {
                    flags.push(syn::Expr::Path(
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_DICT},
                    ));
                }
                param => {
                    println!("Unsupported parameter: {}", param);
                }
            },

            // Match a key/value flag
            syn::Expr::Assign(ref ass) => {
                let key = match *ass.left {
                    syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                        exp.path.segments.first().unwrap().value().ident.to_string()
                    }
                    _ => panic!("could not parse argument: {:?}", ass),
                };

                match key.as_str() {
                    "freelist" => {
                        // TODO: check if int literal
                        params.insert("freelist", *ass.right.clone());
                    }
                    "name" => match *ass.right {
                        syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                            params.insert("name", exp.clone().into());
                        }
                        _ => println!("Wrong 'name' format: {:?}", *ass.right),
                    },
                    "base" => match *ass.right {
                        syn::Expr::Path(ref exp) => {
                            base = syn::TypePath {
                                path: exp.path.clone(),
                                qself: None,
                            };
                        }
                        _ => println!("Wrong 'base' format: {:?}", *ass.right),
                    },
                    _ => {
                        println!("Unsupported parameter: {:?}", key);
                    }
                }
            }

            _ => panic!("could not parse arguments"),
        }
    }

    (params, flags, base)
}
