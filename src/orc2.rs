///! everything related to llvm::orc::LLJIT
use core::fmt;
use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt::{Formatter,Debug};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use llvm_sys::error::LLVMGetErrorMessage;
use llvm_sys::orc2::lljit::{LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT as DisposeLLJIT, LLVMOrcDisposeLLJITBuilder, LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITBuilderRef, LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITLookup, LLVMOrcLLJITMangleAndIntern, LLVMOrcLLJITRef};
use llvm_sys::orc2::{LLVMJITEvaluatedSymbol, LLVMJITSymbolFlags, LLVMOrcAbsoluteSymbols, LLVMOrcCSymbolMapPair, LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule, LLVMOrcJITDylibDefine, LLVMOrcJITDylibRef};
use crate::module::Module;
use crate::support::{to_c_str,LLVMString};
use crate::values::GlobalValue;

/// A light wrapper around llvm::orc::LLJit.
/// Should be constructed from [crate::module::Module::crate_lljit_engine]
#[derive(Debug, PartialEq, Eq)]
pub struct LLJITExecutionEngine<'ctx>(LLVMOrcLLJITRef,PhantomData<&'ctx ()>);

impl Drop for LLJITExecutionEngine<'_> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { DisposeLLJIT(self.0) };
        }
    }
}

impl<'ctx> LLJITExecutionEngine<'ctx> {
    /// Try to load a function from the execution engine.
    ///
    /// The [`UnsafeFunctionPointer`] trait is designed so only `unsafe extern
    /// "C"` functions can be retrieved via the `get_function()` method. If you
    /// get funny type errors then it's probably because you have specified the
    /// wrong calling convention or forgotten to specify the retrieved function
    /// as `unsafe`.
    ///
    /// # Examples
    ///
    ///
    /// ```rust,no_run
    /// # use inkwell::targets::{InitializationConfig, Target};
    /// # use inkwell::context::Context;
    /// # use inkwell::OptimizationLevel;
    /// # Target::initialize_native(&InitializationConfig::default()).unwrap();
    /// let context = Context::create();
    /// let module = context.create_module("test");
    /// let builder = context.create_builder();
    ///
    /// // Set up the function signature
    /// let double = context.f64_type();
    /// let sig = double.fn_type(&[], false);
    ///
    /// // Add the function to our module
    /// let f = module.add_function("test_fn", sig, None);
    /// let b = context.append_basic_block(f, "entry");
    /// builder.position_at_end(b);
    ///
    /// // Insert a return statement
    /// let ret = double.const_float(64.0);
    /// builder.build_return(Some(&ret)).unwrap();
    ///
    /// // create the JIT engine
    /// let mut ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// // fetch our JIT'd function and execute it
    /// unsafe {
    ///     let test_fn = ee.get_function::<unsafe extern "C" fn() -> f64>("test_fn").unwrap();
    ///     let return_value = test_fn.call();
    ///     assert_eq!(return_value, 64.0);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to ensure they call the function with
    /// the correct signature and calling convention.
    ///
    /// The `JitFunction` wrapper ensures a function won't accidentally outlive the
    /// execution engine it came from, but adding functions after calling this
    /// method *may* invalidate the function pointer.
    ///
    /// [`UnsafeFunctionPointer`]: trait.UnsafeFunctionPointer.html
    pub unsafe fn get_function<F:crate::execution_engine::UnsafeFunctionPointer>(&self,name:impl AsRef<str>) -> Result<LLJITFunction<'ctx,F>,LLVMString> {
        let name = to_c_str(name.as_ref());
        let mut address = MaybeUninit::uninit();
        let err = LLVMOrcLLJITLookup(self.0, address.as_mut_ptr(), name.as_ptr());
        if !err.is_null() {
            let msg = LLVMGetErrorMessage(err);
            Err(LLVMString::new(msg))
        } else {
            let address = address.assume_init();
            if address == 0 {
                Err(LLVMString::create_from_str("Unknown Error in getting a jit function."))
            } else {
                Ok(LLJITFunction::<'ctx,F> { addr: address, _f: Default::default() })
            }
        }
    }

    /// Maps the specified name to an address.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::targets::{InitializationConfig, Target};
    /// use inkwell::context::Context;
    /// use inkwell::OptimizationLevel;
    ///
    /// Target::initialize_native(&InitializationConfig::default()).unwrap();
    ///
    /// extern fn sumf(a: f64, b: f64) -> f64 {
    ///     a + b
    /// }
    ///
    /// let context = Context::create();
    /// let module = context.create_module("test");
    /// let builder = context.create_builder();
    ///
    /// let ft = context.f64_type();
    /// let fnt = ft.fn_type(&[], false);
    ///
    /// let f = module.add_function("test_fn", fnt, None);
    /// let b = context.append_basic_block(f, "entry");
    ///
    /// builder.position_at_end(b);
    ///
    /// let extf = module.add_function("sumf", ft.fn_type(&[ft.into(), ft.into()], false), None);
    ///
    /// let argf = ft.const_float(64.);
    /// let call_site_value = builder.build_call(extf, &[argf.into(), argf.into()], "retv").unwrap();
    /// let retv = call_site_value.try_as_basic_value().left().unwrap().into_float_value();
    ///
    /// builder.build_return(Some(&retv)).unwrap();
    ///
    /// let mut ee = module.create_lljit_engine().unwrap();
    /// ee.add_global_mapping("sumf", sumf as usize);
    ///
    /// let result = unsafe {
    ///     let fun = ee.get_function::<unsafe extern "C" fn()>("test_fn");
    ///     fun.call()
    /// };
    ///
    /// assert_eq!(result, 128.);
    /// ```
    pub fn add_global_mapping(&self, name : impl AsRef<str>, addr : usize) -> Result<(),LLVMString> {
        let name = to_c_str(name.as_ref());
        self.add_global_mapping_impl(name, addr)
    }

    /// A grouped version of [Self::add_global_mapping]
    pub fn add_global_mappings(&self, mappings:&[(&str,usize)]) -> Result<(),LLVMString> {
        self.add_global_mappings_impl(mappings.iter().map(|(name,addr)| (to_c_str(name),*addr)))
    }

    fn add_global_mapping_impl(&self,name:Cow<'_,CStr>, addr:usize) -> Result<(),LLVMString> {
        self.add_global_mappings_impl(std::iter::once((name,addr)))
    }

    fn add_global_mappings_impl<'a>(&self,mappings:impl Iterator<Item = (Cow<'a,CStr>,usize)>)->Result<(),LLVMString> {
        unsafe {
            let jd = self.get_main_dylib();
            let mut mappings = mappings.map(|(name,addr)| {
                LLVMOrcCSymbolMapPair {
                    Name:LLVMOrcLLJITMangleAndIntern(self.0, name.as_ptr()),
                    Sym:LLVMJITEvaluatedSymbol {
                        Address:addr as u64,
                        Flags: LLVMJITSymbolFlags {
                            //todo? find the correct flags?
                            GenericFlags:0,
                            TargetFlags:0
                        }
                    }
                }
            }).collect::<Vec<_>>();
            let mu = LLVMOrcAbsoluteSymbols(mappings.as_mut_ptr(), mappings.len());
            let err = LLVMOrcJITDylibDefine(jd, mu);
            if !err.is_null() {
                let msg = LLVMGetErrorMessage(err);
                Err(LLVMString::new(msg))
            } else {
                Ok(())
            }
        }
    }


    /// Maps a global value to an address
    /// Restricted to [crate::values::GlobalValue] as that's all that is supported for remapping by the LLJIT engine.
    /// # Example
    /// ```no_run
    /// # use inkwell::targets::{InitializationConfig, Target};
    /// # use inkwell::context::Context;
    /// # use inkwell::OptimizationLevel;
    ///
    /// Target::initialize_native(&InitializationConfig::default()).unwrap();
    ///
    /// extern fn sumf(a: f64, b: f64) -> f64 {
    ///     a + b
    /// }
    ///
    /// let context = Context::create();
    /// let module = context.create_module("test");
    /// let builder = context.create_builder();
    ///
    /// let ft = context.f64_type();
    /// let fnt = ft.fn_type(&[], false);
    ///
    /// let f = module.add_function("test_fn", fnt, None);
    /// let b = context.append_basic_block(f, "entry");
    ///
    /// builder.position_at_end(b);
    ///
    /// let extf = module.add_function("sumf", ft.fn_type(&[ft.into(), ft.into()], false), None);
    ///
    /// let argf = ft.const_float(64.);
    /// let call_site_value = builder.build_call(extf, &[argf.into(), argf.into()], "retv").unwrap();
    /// let retv = call_site_value.try_as_basic_value().left().unwrap().into_float_value();
    ///
    /// builder.build_return(Some(&retv)).unwrap();
    ///
    /// let mut ee = module.create_lljit_engine().unwrap();
    /// ee.add_global_mapping_by_value(&extf.as_global_value(), sumf as usize);
    ///
    /// let result = unsafe {
    ///     let fun = ee.get_function::<unsafe extern "C" fn()>("test_fn");
    ///     fun.call()
    /// };
    ///
    /// assert_eq!(result, 128.);
    /// ```
    pub fn add_global_mapping_by_value(&self,v:&GlobalValue<'ctx>, addr:usize) -> Result<(),LLVMString> {
        let name= v.get_name();
        self.add_global_mapping_impl(Cow::from(name), addr)
    }

    fn get_main_dylib(&self) -> LLVMOrcJITDylibRef {
        unsafe { LLVMOrcLLJITGetMainJITDylib(self.0) }
    }

    /// Adds a module to the engine.
    /// It takes ownership as it is illegal to modify the module once it has been added.
    /// # Example
    /// ```
    /// # use inkwell::targets::{InitializationConfig, Target};
    /// # use inkwell::context::Context;
    /// let ctx = Context::create();
    /// let base_module = ctx.create_module("base");
    /// let ee = base_module.create_lljit_engine().unwrap();
    /// 
    /// let new_module = ctx.create_module("new");
    /// let fun_ty = ctx.i32_type().fn_type(&[],false);
    /// let fun = new_module.add_function("fun",fun_ty,None);
    /// let builder = ctx.create_builder();
    /// let bb = ctx.append_basic_block(fun,"entry");
    /// builder.position_at_end(bb);
    /// let retv = ctx.i32_type().const_int(3,false);
    /// builder.build_return(Some(&retv));
    /// ee.add_module(new_module);
    /// unsafe {
    ///    let fun = ee.get_function::<unsafe extern "C" fn() -> i32>("fun").unwrap();
    ///    let result = fun.call();
    ///    assert_eq!(result,3);
    /// }
    /// ```
    pub fn add_module(&self, m : Module<'ctx>) -> Result<(),LLVMString> {
        unsafe {
            let tsc = LLVMOrcCreateNewThreadSafeContext();
            let tsm = LLVMOrcCreateNewThreadSafeModule(m.as_mut_ptr(), tsc);
            let dylib = self.get_main_dylib();
            let err = LLVMOrcLLJITAddLLVMIRModule(self.0, dylib, tsm);
            if !err.is_null() {
                let msg = LLVMGetErrorMessage(err);
                Err(LLVMString::new(msg))
            } else {
                Ok(())
            }
        }
    }
}


/// A wrapper around a function pointer which ensures the function being pointed
/// to doesn't accidentally outlive its execution engine.
#[derive(Clone)]
pub struct LLJITFunction<'ctx,F> {
    addr : u64,
    _f : PhantomData<&'ctx F>,
}

impl<F> Debug for LLJITFunction<'_,F> {
    fn fmt(&self, f:&mut Formatter) -> fmt::Result {
        f.debug_tuple("LLJITFunction").field(&"<unnamed>").finish()
    }
}

macro_rules! impl_unsafe_fn {
    (@recurse $first:ident $( , $rest:ident )*) => {
        impl_unsafe_fn!($( $rest ),*);
    };

    (@recurse) => {};

    ($( $param:ident ),*) => {
        impl<Output, $( $param ),*> LLJITFunction<'_, unsafe extern "C" fn($( $param ),*) -> Output> {
            /// This method allows you to call the underlying function while making
            /// sure that the backing storage is not dropped too early and
            /// preserves the `unsafe` marker for any calls.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub unsafe fn call(&self, $( $param: $param ),*) -> Output {
                let addr = ::std::mem::transmute::<u64,unsafe extern "C" fn($( $param ),*) -> Output>(self.addr);
                (addr)($( $param ),*)
            }
        }

        impl_unsafe_fn!(@recurse $( $param ),*);
    };
}

impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M);

#[derive(Debug)]
pub(crate) struct LLJITBuilder<'ctx>(LLVMOrcLLJITBuilderRef,PhantomData<&'ctx ()>);
impl Drop for LLJITBuilder<'_> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { LLVMOrcDisposeLLJITBuilder(self.0) };
        }
    }
}

impl<'ctx> LLJITBuilder<'ctx> {
    pub(crate) fn new() -> Self {
        Self(unsafe { LLVMOrcCreateLLJITBuilder() },PhantomData::default())
    }

    /// TODO? extra options?

    pub(crate) fn create(self) -> Result<LLJITExecutionEngine<'ctx>,LLVMString> {
        let data = self.1;
        let builder = self.0;
        std::mem::forget(self);
        let mut out = MaybeUninit::uninit();
        let err = unsafe { LLVMOrcCreateLLJIT(out.as_mut_ptr(), builder) };
        if !err.is_null() {
            let msg = unsafe { LLVMGetErrorMessage(err) };
            Err(unsafe { LLVMString::new(msg) })
        } else {
            let out = unsafe { out.assume_init() };
            if out.is_null() {
                Err(LLVMString::create_from_str("LLJIT failed to be constructed for unknown reasons"))
            } else {
                Ok(LLJITExecutionEngine(out, data))
            }
        }
    }
}