/**
 * Complex Kotlin example demonstrating coroutines, sealed classes, 
 * data classes, extension functions, and modern Kotlin features
 * for comprehensive parser testing.
 */

package com.example.coroutines

import kotlinx.coroutines.*
import kotlinx.coroutines.channels.*
import kotlinx.coroutines.flow.*
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

// Sealed class hierarchy
sealed class Result<out T> {
    data class Success<T>(val data: T) : Result<T>()
    data class Error(val exception: Throwable) : Result<Nothing>()
    object Loading : Result<Nothing>()
    
    // Extension functions on sealed class
    fun <R> map(transform: (T) -> R): Result<R> = when (this) {
        is Success -> Success(transform(data))
        is Error -> this
        is Loading -> this
    }
    
    fun getOrNull(): T? = when (this) {
        is Success -> data
        else -> null
    }
}

// Data classes with different features
@JvmRecord
data class User(
    val id: String,
    val name: String,
    val email: String,
    val age: Int = 0,
    val roles: List<String> = emptyList()
) {
    // Secondary constructor
    constructor(id: String, name: String, email: String) : this(id, name, email, 0)
    
    // Custom property with getter
    val isAdult: Boolean
        get() = age >= 18
    
    // Member function
    fun hasRole(role: String): Boolean = roles.contains(role)
    
    companion object {
        const val MIN_AGE = 0
        const val MAX_AGE = 150
        
        fun createGuest(): User = User("guest", "Guest User", "guest@example.com")
        
        // Factory function with validation
        fun create(id: String, name: String, email: String, age: Int): Result<User> {
            return when {
                age !in MIN_AGE..MAX_AGE -> Result.Error(IllegalArgumentException("Invalid age"))
                email.isBlank() -> Result.Error(IllegalArgumentException("Email cannot be blank"))
                else -> Result.Success(User(id, name, email, age))
            }
        }
    }
}

// Enum class with properties and methods
enum class ProcessingStatus(val priority: Int, val description: String) {
    QUEUED(1, "Waiting in queue"),
    PROCESSING(2, "Currently processing"),
    COMPLETED(3, "Successfully completed"),
    FAILED(0, "Processing failed"),
    CANCELLED(-1, "Processing cancelled");
    
    fun isActive(): Boolean = this in setOf(QUEUED, PROCESSING)
    
    companion object {
        fun fromPriority(priority: Int): ProcessingStatus? {
            return values().find { it.priority == priority }
        }
    }
}

// Interface with default methods
interface DataProcessor<T> {
    suspend fun process(data: T): Result<T>
    
    // Default implementation
    suspend fun processWithTimeout(data: T, timeout: Duration = 30.seconds): Result<T> {
        return withTimeoutOrNull(timeout) {
            process(data)
        } ?: Result.Error(TimeoutCancellationException("Processing timed out"))
    }
    
    // Extension property on interface
    val T.isValid: Boolean
        get() = this != null
}

// Generic class with constraints
class AsyncDataProcessor<T : Any>(
    private val processingDelay: Duration = 100.milliseconds,
    private val maxConcurrency: Int = 10
) : DataProcessor<T> {
    
    // Private properties
    private val _status = MutableStateFlow(ProcessingStatus.QUEUED)
    val status: StateFlow<ProcessingStatus> = _status.asStateFlow()
    
    private val semaphore = Semaphore(maxConcurrency)
    private val processingChannel = Channel<ProcessingRequest<T>>(Channel.UNLIMITED)
    
    // Nested data class
    private data class ProcessingRequest<T>(
        val data: T,
        val deferred: CompletableDeferred<Result<T>>
    )
    
    // Initialization block
    init {
        // Start background processor
        GlobalScope.launch {
            processRequests()
        }
    }
    
    override suspend fun process(data: T): Result<T> = withContext(Dispatchers.Default) {
        val deferred = CompletableDeferred<Result<T>>()
        val request = ProcessingRequest(data, deferred)
        
        processingChannel.send(request)
        deferred.await()
    }
    
    // Suspend function with structured concurrency
    private suspend fun processRequests() = coroutineScope {
        processingChannel.consumeAsFlow()
            .collect { request ->
                launch {
                    semaphore.withPermit {
                        _status.value = ProcessingStatus.PROCESSING
                        
                        try {
                            // Simulate processing
                            delay(processingDelay)
                            
                            val result = when {
                                request.data.toString().contains("error") -> {
                                    Result.Error(RuntimeException("Simulated error"))
                                }
                                else -> Result.Success(request.data)
                            }
                            
                            request.deferred.complete(result)
                            _status.value = ProcessingStatus.COMPLETED
                            
                        } catch (e: Exception) {
                            request.deferred.complete(Result.Error(e))
                            _status.value = ProcessingStatus.FAILED
                        }
                    }
                }
            }
    }
    
    // Flow-based processing
    fun processFlow(dataFlow: Flow<T>): Flow<Result<T>> = flow {
        dataFlow.collect { data ->
            emit(process(data))
        }
    }.flowOn(Dispatchers.IO)
    
    // Batch processing with chunking
    suspend fun processBatch(items: List<T>, batchSize: Int = 5): Flow<List<Result<T>>> = flow {
        items.chunked(batchSize).forEach { batch ->
            val results = batch.map { item ->
                async { process(item) }
            }.awaitAll()
            
            emit(results)
        }
    }.flowOn(Dispatchers.Default)
    
    // Channel-based producer
    fun produceResults(scope: CoroutineScope, items: List<T>): ReceiveChannel<Result<T>> =
        scope.produce {
            for (item in items) {
                send(process(item))
            }
        }
    
    companion object {
        // Factory function with reified type
        inline fun <reified T : Any> create(
            delay: Duration = 100.milliseconds,
            concurrency: Int = 10
        ): AsyncDataProcessor<T> {
            return AsyncDataProcessor(delay, concurrency)
        }
    }
}

// Extension functions on existing types
suspend fun <T> List<T>.processParallel(
    processor: DataProcessor<T>,
    concurrency: Int = 10
): List<Result<T>> = coroutineScope {
    val semaphore = Semaphore(concurrency)
    
    map { item ->
        async {
            semaphore.withPermit {
                processor.process(item)
            }
        }
    }.awaitAll()
}

// Extension function with receiver
fun <T> Flow<T>.chunkedWithTimeout(
    size: Int,
    timeout: Duration
): Flow<List<T>> = flow {
    val buffer = mutableListOf<T>()
    var lastEmission = System.currentTimeMillis()
    
    collect { item ->
        buffer.add(item)
        val now = System.currentTimeMillis()
        
        if (buffer.size >= size || (now - lastEmission) >= timeout.inWholeMilliseconds) {
            emit(buffer.toList())
            buffer.clear()
            lastEmission = now
        }
    }
    
    if (buffer.isNotEmpty()) {
        emit(buffer.toList())
    }
}

// Object declaration (Singleton)
object ConfigurationManager {
    private val config = mutableMapOf<String, Any>()
    
    fun set(key: String, value: Any) {
        config[key] = value
    }
    
    inline fun <reified T> get(key: String): T? {
        return config[key] as? T
    }
    
    fun has(key: String): Boolean = config.containsKey(key)
}

// Class with delegation
class UserManager(
    private val dataProcessor: DataProcessor<User>
) : DataProcessor<User> by dataProcessor {
    
    private val userCache = mutableMapOf<String, User>()
    
    // Override with caching
    override suspend fun process(data: User): Result<User> {
        val cached = userCache[data.id]
        if (cached != null) {
            return Result.Success(cached)
        }
        
        return when (val result = dataProcessor.process(data)) {
            is Result.Success -> {
                userCache[data.id] = result.data
                result
            }
            else -> result
        }
    }
    
    // Additional functionality
    suspend fun processUsers(users: List<User>): Flow<Result<User>> = flow {
        users.forEach { user ->
            emit(process(user))
        }
    }
    
    fun clearCache() {
        userCache.clear()
    }
}

// Higher-order functions and lambdas
class AsyncWorkflowBuilder<T> {
    private val steps = mutableListOf<suspend (T) -> T>()
    
    fun step(action: suspend (T) -> T): AsyncWorkflowBuilder<T> {
        steps.add(action)
        return this
    }
    
    fun conditionalStep(
        predicate: (T) -> Boolean,
        action: suspend (T) -> T
    ): AsyncWorkflowBuilder<T> {
        steps.add { data ->
            if (predicate(data)) action(data) else data
        }
        return this
    }
    
    suspend fun execute(initialData: T): Result<T> = try {
        val result = steps.fold(initialData) { data, step ->
            step(data)
        }
        Result.Success(result)
    } catch (e: Exception) {
        Result.Error(e)
    }
    
    companion object {
        fun <T> workflow(
            builder: AsyncWorkflowBuilder<T>.() -> Unit
        ): AsyncWorkflowBuilder<T> {
            return AsyncWorkflowBuilder<T>().apply(builder)
        }
    }
}

// Usage example with complex scenarios
suspend fun demonstrateComplexFeatures() = coroutineScope {
    println("=== Kotlin Coroutines and Advanced Features Demo ===")
    
    // Create processor
    val processor = AsyncDataProcessor.create<User>(
        delay = 50.milliseconds,
        concurrency = 5
    )
    
    // Create sample users
    val users = listOf(
        User("1", "Alice", "alice@example.com", 25, listOf("admin", "user")),
        User("2", "Bob", "bob@example.com", 30, listOf("user")),
        User("3", "Charlie", "charlie@example.com", 22, listOf("user", "moderator")),
        User("error", "Error User", "error@example.com", 35) // Will cause error
    )
    
    // Process with different approaches
    
    // 1. Sequential processing
    println("\n1. Sequential Processing:")
    users.forEach { user ->
        val result = processor.process(user)
        println("Processed ${user.name}: ${result::class.simpleName}")
    }
    
    // 2. Parallel processing with extension function
    println("\n2. Parallel Processing:")
    val parallelResults = users.processParallel(processor, concurrency = 3)
    parallelResults.forEachIndexed { index, result ->
        println("User ${index + 1}: ${result::class.simpleName}")
    }
    
    // 3. Flow-based processing
    println("\n3. Flow Processing:")
    users.asFlow()
        .chunkedWithTimeout(2, 200.milliseconds)
        .collect { batch ->
            println("Processing batch of ${batch.size} users")
            batch.forEach { user ->
                val result = processor.process(user)
                println("  ${user.name}: ${result.getOrNull()?.isAdult ?: "error"}")
            }
        }
    
    // 4. Channel-based processing
    println("\n4. Channel Processing:")
    val channel = processor.produceResults(this, users.take(3))
    
    repeat(3) {
        val result = channel.receive()
        println("Received: ${result::class.simpleName}")
    }
    
    // 5. Workflow builder
    println("\n5. Workflow Processing:")
    val workflow = AsyncWorkflowBuilder.workflow<User> {
        step { user -> user.copy(name = user.name.uppercase()) }
        conditionalStep(
            predicate = { it.age < 25 },
            action = { user -> 
                delay(10) // Simulate async work
                user.copy(roles = user.roles + "young_user")
            }
        )
        step { user -> 
            println("Final processing for ${user.name}")
            user
        }
    }
    
    users.take(2).forEach { user ->
        val result = workflow.execute(user)
        when (result) {
            is Result.Success -> println("Workflow completed for ${result.data.name}")
            is Result.Error -> println("Workflow failed: ${result.exception.message}")
            is Result.Loading -> println("Still loading...")
        }
    }
    
    // 6. Batch processing with flows
    println("\n6. Batch Flow Processing:")
    processor.processBatch(users, batchSize = 2)
        .collect { batchResults ->
            val successCount = batchResults.count { it is Result.Success }
            val errorCount = batchResults.count { it is Result.Error }
            println("Batch completed: $successCount successes, $errorCount errors")
        }
    
    // Configuration usage
    ConfigurationManager.set("max_users", 1000)
    ConfigurationManager.set("debug_mode", true)
    
    val maxUsers = ConfigurationManager.get<Int>("max_users")
    val debugMode = ConfigurationManager.get<Boolean>("debug_mode")
    
    println("\nConfiguration: max_users=$maxUsers, debug_mode=$debugMode")
    
    // Status monitoring
    launch {
        processor.status.collect { status ->
            println("Processor status: ${status.description}")
        }
    }
    
    delay(1.seconds) // Let everything complete
    println("\nDemo completed!")
}

// Main function
suspend fun main() {
    try {
        demonstrateComplexFeatures()
    } catch (e: Exception) {
        println("Demo failed with error: ${e.message}")
        e.printStackTrace()
    }
}