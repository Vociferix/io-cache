# io-cache
This is an **EXTREMELY** early code base. It does not compile, and is not really useful in any way, at 
this time. I am making it public because it causes the rust compiler to panic. See below for more
information.

The eventual purpose of this library is to provide a customizable `IOCache` struct that holds a type
that implements `Read + Seek` and optionally `Write`, which is generally a `File`, and reserves memory
for caching. The idea is, through the use of macros and unusual use of rust generics, the user can
easily build a use case optimized cache. The configurations will allow for various caching techniques
to be tuned and combined as needed.

## Example Customization
```rust
// The following is a potential example and is subject to change
cache_config! {
    config MyConfig {
        block_size: 256,
        associativity: NWay(8),
        replacement: LRU,
        blocks_per_fetch: 2,
        write_stratgey: WriteBack,
        async_write: true,
    }
}

const CACHE_MEM: usize = 1024 * 1024 * 1024; // 1 GiB

let file = std::fs::File::open("/path/to/file")?;

let my_cache: IOCache<MyConfig<std::io::File>> = IOCache::new_strict(file, CACHE_MEM);
```

# rustc panic
Below is the compiler and build information. Based on the information, it appears the problem has to do
with rust's in-progress `const` semantics, although I might not know what I'm talking about. I just know
that my use of `const` and generics in this library is fairly atypical (or at least I believe it to be),
so it makes sense to me that the problem lies there.

I apologize for the state of this code base. It has no comments, no tests, and is full of experimentation
code to see what will compile and what won't. I did at least clean up all the warnings before posting
this, to make life a bit easier. It is a one-man hobby project, and I am still relatively new to rust. A
lot of the code here is my personal translation of C++ meta programming over to rust.

A note to keep in mind: Everything compiled until I added the threading code for `AsyncIO` in
src/detail/io.rs, which I find interesting since it appears to be complaining about something that
previously compiled with no problem, and is unrelated to the threading code.

Also note that if the compiler did not panic, there is a good chance there is something actually wrong
with the code that would fail to compile, which would be something in src/detail/io.rs.

```
$ rustc --version
rustc 1.33.0 (2aa4c46cf 2019-02-28)
```

```
$ rustup show
Default host: x86_64-unknown-linux-gnu

installed toolchains
--------------------

stable-x86_64-unknown-linux-gnu (default)
beta-x86_64-unknown-linux-gnu
nightly-x86_64-unknown-linux-gnu

active toolchain
----------------

stable-x86_64-unknown-linux-gnu (default)
rustc 1.33.0 (2aa4c46cf 2019-02-28)

```

```
$ RUST_BACKTRACE=FULL cargo build --verbose
   Compiling io-cache v0.1.0 (/home/jack/devel/io-cache)
     Running `rustc --edition=2018 --crate-name io_cache src/lib.rs --color always --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=531453a790193d18 -C extra-filename=-531453a790193d18 --out-dir /home/jack/devel/io-cache/target/debug/deps -C incremental=/home/jack/devel/io-cache/target/debug/incremental -L dependency=/home/jack/devel/io-cache/target/debug/deps`
error: internal compiler error: src/librustc/ty/subst.rs:480: Type parameter `R/#1` (R/1) out of range when substituting (root type=Some(detail::set::NWaySet<L, R, Block, Blocks>)) substs=[detail::set::NWaySet<L, R, Block, Blocks>]

thread 'rustc' panicked at 'Box<Any>', src/librustc_errors/lib.rs:526:9
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
stack backtrace:
   0: std::sys::unix::backtrace::tracing::imp::unwind_backtrace
             at src/libstd/sys/unix/backtrace/tracing/gcc_s.rs:39
   1: std::sys_common::backtrace::_print
             at src/libstd/sys_common/backtrace.rs:70
   2: std::panicking::default_hook::{{closure}}
             at src/libstd/sys_common/backtrace.rs:58
             at src/libstd/panicking.rs:200
   3: std::panicking::default_hook
             at src/libstd/panicking.rs:215
   4: rustc::util::common::panic_hook
   5: std::panicking::rust_panic_with_hook
             at src/libstd/panicking.rs:482
   6: std::panicking::begin_panic
   7: rustc_errors::Handler::span_bug
   8: rustc::util::bug::opt_span_bug_fmt::{{closure}}
   9: rustc::ty::context::tls::with_opt::{{closure}}
  10: rustc::ty::context::tls::with_context_opt
  11: rustc::ty::context::tls::with_opt
  12: rustc::util::bug::opt_span_bug_fmt
  13: rustc::util::bug::span_bug_fmt
  14: <rustc::ty::subst::SubstFolder<'a, 'gcx, 'tcx> as rustc::ty::fold::TypeFolder<'gcx, 'tcx>>::fold_ty
  15: <smallvec::SmallVec<A> as core::iter::traits::FromIterator<<A as smallvec::Array>::Item>>::from_iter
  16: rustc::ty::fold::TypeFoldable::fold_with
  17: rustc::ty::structural_impls::<impl rustc::ty::fold::TypeFoldable<'tcx> for &'tcx rustc::ty::TyS<'tcx>>::super_fold_with
  18: <rustc::ty::subst::SubstFolder<'a, 'gcx, 'tcx> as rustc::ty::fold::TypeFolder<'gcx, 'tcx>>::fold_ty
  19: <smallvec::SmallVec<A> as core::iter::traits::FromIterator<<A as smallvec::Array>::Item>>::from_iter
  20: rustc::ty::fold::TypeFoldable::fold_with
  21: rustc::traits::codegen::<impl rustc::ty::context::TyCtxt<'a, 'tcx, 'tcx>>::subst_and_normalize_erasing_regions
  22: <rustc_mir::interpret::eval_context::EvalContext<'a, 'mir, 'tcx, M>>::resolve
  23: rustc_mir::interpret::step::<impl rustc_mir::interpret::eval_context::EvalContext<'a, 'mir, 'tcx, M>>::run
  24: rustc_mir::const_eval::eval_body_using_ecx
  25: rustc_mir::const_eval::const_eval_raw_provider
  26: rustc::ty::query::__query_compute::const_eval_raw
  27: rustc::ty::query::<impl rustc::ty::query::config::QueryAccessors<'tcx> for rustc::ty::query::queries::const_eval_raw<'tcx>>::compute
  28: rustc::dep_graph::graph::DepGraph::with_task_impl
  29: rustc::ty::query::plumbing::<impl rustc::ty::context::TyCtxt<'a, 'gcx, 'tcx>>::try_get_with
  30: <rustc_mir::interpret::eval_context::EvalContext<'a, 'mir, 'tcx, M>>::const_eval_raw
  31: rustc_mir::interpret::operand::<impl rustc_mir::interpret::eval_context::EvalContext<'a, 'mir, 'tcx, M>>::const_value_to_op
  32: rustc_mir::const_eval::lazy_const_to_op
  33: rustc_mir::transform::const_prop::ConstPropagator::eval_constant
  34: <rustc_mir::transform::const_prop::ConstPropagator<'b, 'a, 'tcx> as rustc::mir::visit::Visitor<'tcx>>::visit_statement
  35: <rustc_mir::transform::const_prop::ConstProp as rustc_mir::transform::MirPass>::run_pass
  36: rustc_mir::transform::run_passes::{{closure}}
  37: rustc_mir::transform::run_passes
  38: rustc_mir::transform::optimized_mir
  39: rustc::ty::query::__query_compute::optimized_mir
  40: rustc::ty::query::<impl rustc::ty::query::config::QueryAccessors<'tcx> for rustc::ty::query::queries::optimized_mir<'tcx>>::compute
  41: rustc::dep_graph::graph::DepGraph::with_task_impl
  42: rustc::ty::query::plumbing::<impl rustc::ty::context::TyCtxt<'a, 'gcx, 'tcx>>::try_get_with
  43: rustc_metadata::encoder::<impl rustc_metadata::isolated_encoder::IsolatedEncoder<'a, 'b, 'tcx>>::encode_optimized_mir
  44: rustc_metadata::encoder::<impl rustc_metadata::isolated_encoder::IsolatedEncoder<'a, 'b, 'tcx>>::encode_info_for_impl_item
  45: rustc::dep_graph::graph::DepGraph::with_ignore
  46: rustc_metadata::encoder::<impl rustc_metadata::index_builder::IndexBuilder<'a, 'b, 'tcx>>::encode_addl_info_for_item
  47: rustc::hir::Crate::visit_all_item_likes
  48: rustc_metadata::encoder::encode_metadata
  49: rustc_metadata::cstore_impl::<impl rustc::middle::cstore::CrateStore for rustc_metadata::cstore::CStore>::encode_metadata
  50: rustc::ty::context::TyCtxt::encode_metadata
  51: <rustc_codegen_llvm::LlvmCodegenBackend as rustc_codegen_ssa::traits::backend::ExtraBackendMethods>::write_metadata
  52: rustc::util::common::time
  53: rustc_codegen_ssa::base::codegen_crate
  54: <rustc_codegen_llvm::LlvmCodegenBackend as rustc_codegen_utils::codegen_backend::CodegenBackend>::codegen_crate
  55: rustc::util::common::time
  56: rustc_driver::driver::phase_4_codegen
  57: rustc_driver::driver::compile_input::{{closure}}
  58: <std::thread::local::LocalKey<T>>::with
  59: rustc::ty::context::TyCtxt::create_and_enter
  60: rustc_driver::driver::compile_input
  61: rustc_driver::run_compiler_with_pool
  62: <scoped_tls::ScopedKey<T>>::set
  63: rustc_driver::run_compiler
  64: <scoped_tls::ScopedKey<T>>::set
query stack during panic:
#0 [const_eval_raw] const-evaluating `<detail::set::NWaySet<L, R, Block, Blocks> as detail::set::Set>::STATIC_META_MEM`
   --> src/detail/set.rs:206:32
    |
206 |       const MEM_PER_SET: usize = (Block::LEN * Blocks::LEN)
    |  ________________________________^
207 | |         + NWaySet::<L, R, Block, Blocks>::STATIC_META_MEM
    | |_________________________________________________________^
#1 [optimized_mir] processing `<detail::set::NWaySets<L, R, Block, Blocks, S>>::MEM_PER_SET`
end of query stack
error: aborting due to previous error


note: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md#bug-reports

note: rustc 1.33.0 (2aa4c46cf 2019-02-28) running on x86_64-unknown-linux-gnu

note: compiler flags: -C debuginfo=2 -C incremental --crate-type lib

note: some of the compiler flags provided by cargo are hidden

error: Could not compile `io-cache`.

Caused by:
  process didn't exit successfully: `rustc --edition=2018 --crate-name io_cache src/lib.rs --color always --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=531453a790193d18 -C extra-filename=-531453a790193d18 --out-dir /home/jack/devel/io-cache/target/debug/deps -C incremental=/home/jack/devel/io-cache/target/debug/incremental -L dependency=/home/jack/devel/io-cache/target/debug/deps` (exit code: 101)

```