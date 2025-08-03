use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use code_context_graph_parser::language::registry::ParserRegistry;
use code_context_graph_core::Language;
use std::fs;
use std::path::PathBuf;
use code_context_graph_parser::visitor::base::ASTVisitor;
use code_context_graph_parser::test_utils::TestUtils;

/// Performance benchmarks for parsing operations
/// These benchmarks measure parsing speed, memory usage, and scalability

fn get_fixture_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(relative_path);
    path
}

fn benchmark_java_parsing(c: &mut Criterion) {
    let fixture_path = get_fixture_path("java/complex_inheritance.java");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Java fixture");
    
    let mut group = c.benchmark_group("java_parsing");
    
    // Benchmark raw parsing speed
    group.bench_function("complex_inheritance", |b| {
        b.iter(|| {
            let result = TestUtils::parse_source(black_box(&source), black_box(Language::Java));
            black_box(result)
        })
    });
    
    // Benchmark with different source sizes
    let small_java = "class Simple { void method() {} }";
    let medium_java = &source[..source.len() / 2]; // Half of the complex example
    let large_java = source.repeat(3); // Triple the size
    
    for (name, code) in [
        ("small", small_java),
        ("medium", medium_java),
        ("large", &large_java),
    ] {
        group.bench_with_input(
            BenchmarkId::new("size_scaling", name),
            &code,
            |b, code| {
                b.iter(|| {
                    let result = TestUtils::parse_source(black_box(code), black_box(Language::Java));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_python_parsing(c: &mut Criterion) {
    let fixture_path = get_fixture_path("python/decorators_example.py");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Python fixture");
    
    let mut group = c.benchmark_group("python_parsing");
    
    group.bench_function("decorators_example", |b| {
        b.iter(|| {
            let result = TestUtils::parse_source(black_box(&source), black_box(Language::Python));
            black_box(result)
        })
    });
    
    // Benchmark specific Python features
    let async_code = r#"
import asyncio

async def complex_async_function():
    await asyncio.sleep(1)
    async with some_context():
        async for item in async_generator():
            yield await process_item(item)

@decorator
@another_decorator(param=value)
class DecoratedClass:
    @property
    def decorated_property(self):
        return self._value
"#;
    
    group.bench_function("async_features", |b| {
        b.iter(|| {
            let result = TestUtils::parse_source(black_box(async_code), black_box(Language::Python));
            black_box(result)
        })
    });
    
    group.finish();
}

fn benchmark_javascript_parsing(c: &mut Criterion) {
    let fixture_path = get_fixture_path("javascript/modern_es6.js");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read JavaScript fixture");
    
    let mut group = c.benchmark_group("javascript_parsing");
    
    group.bench_function("modern_es6", |b| {
        b.iter(|| {
            let result = TestUtils::parse_source(black_box(&source), black_box(Language::JavaScript));
            black_box(result)
        })
    });
    
    // Benchmark different JS patterns
    let class_heavy = r#"
class BaseClass {
    constructor() { this.value = 0; }
    method1() { return this.value * 2; }
    async method2() { return await fetch('/api'); }
    static method3() { return new BaseClass(); }
}

class DerivedClass extends BaseClass {
    constructor(value) { super(); this.value = value; }
    method1() { return super.method1() + 1; }
}
"#;
    
    let function_heavy = r#"
const func1 = () => ({ key: 'value' });
const func2 = async (param) => await processAsync(param);
const func3 = function* generator() { yield* [1, 2, 3]; };
const func4 = (a, b, ...rest) => [a, b, ...rest];

function regularFunction(callback) {
    return callback ? callback() : null;
}
"#;
    
    for (name, code) in [
        ("class_heavy", class_heavy),
        ("function_heavy", function_heavy),
    ] {
        group.bench_with_input(
            BenchmarkId::new("pattern", name),
            &code,
            |b, code| {
                b.iter(|| {
                    let result = TestUtils::parse_source(black_box(code), black_box(Language::JavaScript));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_kotlin_parsing(c: &mut Criterion) {
    let fixture_path = get_fixture_path("kotlin/coroutines_example.kt");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Kotlin fixture");
    
    let mut group = c.benchmark_group("kotlin_parsing");
    
    group.bench_function("coroutines_example", |b| {
        b.iter(|| {
            let result = TestUtils::parse_source(black_box(&source), black_box(Language::Kotlin));
            black_box(result)
        })
    });
    
    // Benchmark Kotlin-specific features
    let data_classes = r#"
data class User(val id: String, val name: String, val email: String)
data class Product(val id: Long, val name: String, val price: Double)
data class Order(val id: String, val user: User, val products: List<Product>)

sealed class Result<out T> {
    data class Success<T>(val data: T) : Result<T>()
    data class Error(val exception: Throwable) : Result<Nothing>()
    object Loading : Result<Nothing>()
}
"#;
    
    let coroutines = r#"
suspend fun processData(data: List<String>): List<String> = coroutineScope {
    data.map { item ->
        async(Dispatchers.IO) {
            delay(100)
            item.uppercase()
        }
    }.awaitAll()
}

class DataProcessor {
    suspend fun process(items: Flow<String>): Flow<String> = flow {
        items.collect { item ->
            emit(transformItem(item))
        }
    }.flowOn(Dispatchers.Default)
}
"#;
    
    for (name, code) in [
        ("data_classes", data_classes),
        ("coroutines", coroutines),
    ] {
        group.bench_with_input(
            BenchmarkId::new("feature", name),
            &code,
            |b, code| {
                b.iter(|| {
                    let result = TestUtils::parse_source(black_box(code), black_box(Language::Kotlin));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_language_comparison(c: &mut Criterion) {
    // Compare parsing performance across languages with similar complexity
    let mut group = c.benchmark_group("language_comparison");
    
    let java_class = r#"
public class Calculator {
    private double value;
    
    public Calculator(double initialValue) {
        this.value = initialValue;
    }
    
    public double add(double x) {
        this.value += x;
        return this.value;
    }
    
    public double multiply(double x) {
        this.value *= x;
        return this.value;
    }
}
"#;
    
    let python_class = r#"
class Calculator:
    def __init__(self, initial_value):
        self.value = initial_value
    
    def add(self, x):
        self.value += x
        return self.value
    
    def multiply(self, x):
        self.value *= x
        return self.value
"#;
    
    let javascript_class = r#"
class Calculator {
    constructor(initialValue) {
        this.value = initialValue;
    }
    
    add(x) {
        this.value += x;
        return this.value;
    }
    
    multiply(x) {
        this.value *= x;
        return this.value;
    }
}
"#;
    
    let kotlin_class = r#"
class Calculator(private var value: Double) {
    fun add(x: Double): Double {
        value += x
        return value
    }
    
    fun multiply(x: Double): Double {
        value *= x
        return value
    }
}
"#;
    
    for (language, code) in [
        (Language::Java, java_class),
        (Language::Python, python_class),
        (Language::JavaScript, javascript_class),
        (Language::Kotlin, kotlin_class),
    ] {
        group.bench_with_input(
            BenchmarkId::new("simple_class", format!("{:?}", language)),
            &(language, code),
            |b, (lang, code)| {
                b.iter(|| {
                    let result = TestUtils::parse_source(black_box(code), black_box(*lang));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_parser_registry_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_registry");
    
    let registry = ParserRegistry::new();
    let test_code = "class Test { void method() {} }";
    
    // Benchmark registry overhead
    group.bench_function("registry_parse", |b| {
        b.iter(|| {
            let result = registry.parse(black_box(test_code), black_box(Language::Java));
            black_box(result)
        })
    });
    
    // Benchmark direct parser vs registry
    group.bench_function("direct_vs_registry", |b| {
        b.iter(|| {
            // This would compare direct parser instantiation vs registry lookup
            let result = TestUtils::parse_source(black_box(test_code), black_box(Language::Java));
            black_box(result)
        })
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    // These benchmarks focus on memory efficiency
    let mut group = c.benchmark_group("memory_usage");
    
    // Create files of increasing size
    let small_file = "class A {}".repeat(10);
    let medium_file = "class A { void method() { System.out.println(\"test\"); } }".repeat(100);
    let large_file = "class A { void method() { System.out.println(\"test\"); } }".repeat(1000);
    
    for (name, code) in [
        ("small_10_classes", &small_file),
        ("medium_100_classes", &medium_file),
        ("large_1000_classes", &large_file),
    ] {
        group.bench_with_input(
            BenchmarkId::new("memory_scaling", name),
            &code,
            |b, code| {
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();
                    
                    for _ in 0..iters {
                        let result = TestUtils::parse_source(black_box(code), black_box(Language::Java));
                        black_box(result);
                        
                        // Force garbage collection between iterations to get clean measurements
                        // Note: Rust doesn't have explicit GC, but this simulates memory pressure
                        std::hint::black_box(vec![0u8; 1024]); // Allocate some memory
                    }
                    
                    start.elapsed()
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    let valid_code = "class Valid { void method() {} }";
    let invalid_code = "class Invalid { void method( {} }"; // Missing closing paren
    let severely_broken = "class ??? invalid syntax everywhere !!!";
    
    // Benchmark parsing of valid vs invalid code
    for (name, code) in [
        ("valid", valid_code),
        ("invalid", invalid_code),
        ("severely_broken", severely_broken),
    ] {
        group.bench_with_input(
            BenchmarkId::new("error_recovery", name),
            &code,
            |b, code| {
                b.iter(|| {
                    // Parse and ignore errors - we're measuring recovery performance
                    let result = TestUtils::parse_source(black_box(code), black_box(Language::Java));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark AST processing performance
fn benchmark_ast_processing(c: &mut Criterion) {
    let fixture_path = get_fixture_path("java/complex_inheritance.java");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Java fixture");
    
    // Pre-parse the AST so we're only measuring processing time
    let ast = TestUtils::parse_source(&source, Language::Java)
        .expect("Failed to parse Java source");
    
    let mut group = c.benchmark_group("ast_processing");
    
    // Benchmark entity extraction
    group.bench_function("entity_extraction", |b| {
        b.iter(|| {
            let mut context = TestUtils::create_test_context(Language::Java, &source, "test.java");
            let mut extractor = code_context_graph_parser::visitor::entity_extractor::EntityExtractor::new();
            let result = extractor.visit_ast(black_box(&ast), black_box(&mut context));
            black_box(result)
        })
    });
    
    // Benchmark relation extraction
    group.bench_function("relation_extraction", |b| {
        b.iter(|| {
            let mut context = TestUtils::create_test_context(Language::Java, &source, "test.java");
            let mut extractor = code_context_graph_parser::visitor::relation_extractor::RelationExtractor::new();
            let result = extractor.visit_ast(black_box(&ast), black_box(&mut context));
            black_box(result)
        })
    });
    
    // Benchmark metadata collection
    group.bench_function("metadata_collection", |b| {
        b.iter(|| {
            let mut context = TestUtils::create_test_context(Language::Java, &source, "test.java");
            let mut collector = code_context_graph_parser::visitor::metadata_collector::MetadataCollector::new();
            let result = collector.visit_ast(black_box(&ast), black_box(&mut context));
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    parsing_benchmarks,
    benchmark_java_parsing,
    benchmark_python_parsing,
    benchmark_javascript_parsing,
    benchmark_kotlin_parsing,
    benchmark_language_comparison,
    benchmark_parser_registry_performance,
    benchmark_memory_usage,
    benchmark_error_handling,
    benchmark_ast_processing
);

criterion_main!(parsing_benchmarks);