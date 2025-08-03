/**
 * Modern JavaScript ES6+ example demonstrating advanced features
 * for comprehensive parser testing including classes, async/await,
 * modules, generators, and modern syntax.
 */

// Import statements
import { EventEmitter } from 'events';
import fs from 'fs/promises';
import path from 'path';

// Default import
import React from 'react';

// Named and default imports
import axios, { AxiosResponse, AxiosError } from 'axios';

// Dynamic import example
const { createHash } = await import('crypto');

// Constants and symbols
const API_BASE_URL = 'https://api.example.com';
const CONFIG_TIMEOUT = 30000;
const SECRET_KEY = Symbol('secretKey');

// Enums using objects
const Status = Object.freeze({
    PENDING: 'pending',
    PROCESSING: 'processing', 
    COMPLETED: 'completed',
    FAILED: 'failed'
});

// Type definitions using JSDoc
/**
 * @typedef {Object} User
 * @property {string} id - User ID
 * @property {string} name - User name
 * @property {string} email - User email
 * @property {number} age - User age
 * @property {string[]} roles - User roles
 */

/**
 * @typedef {Object} ProcessingResult
 * @property {boolean} success
 * @property {any} data
 * @property {Error?} error
 */

// Modern class with private fields and methods
class DataProcessor extends EventEmitter {
    // Private fields
    #config;
    #cache = new Map();
    #isProcessing = false;
    
    // Static properties
    static DEFAULT_BATCH_SIZE = 100;
    static MAX_RETRIES = 3;
    
    constructor(config = {}) {
        super();
        this.#config = {
            batchSize: DataProcessor.DEFAULT_BATCH_SIZE,
            retries: DataProcessor.MAX_RETRIES,
            timeout: CONFIG_TIMEOUT,
            ...config
        };
        
        // Bind methods to preserve 'this' context
        this.processData = this.processData.bind(this);
        this.handleError = this.handleError.bind(this);
    }
    
    // Getter and setter
    get isProcessing() {
        return this.#isProcessing;
    }
    
    set batchSize(size) {
        if (size > 0) {
            this.#config.batchSize = size;
        }
    }
    
    // Async method with error handling
    async processData(data) {
        if (this.#isProcessing) {
            throw new Error('Processing already in progress');
        }
        
        this.#isProcessing = true;
        this.emit('processing:start', { dataLength: data.length });
        
        try {
            const results = await this.#processBatches(data);
            this.emit('processing:complete', { results });
            return results;
        } catch (error) {
            this.emit('processing:error', error);
            throw error;
        } finally {
            this.#isProcessing = false;
        }
    }
    
    // Private async method with generator
    async #processBatches(data) {
        const results = [];
        
        for await (const batch of this.#createBatches(data)) {
            const batchResult = await this.#processBatch(batch);
            results.push(batchResult);
            
            // Emit progress
            this.emit('processing:progress', {
                processed: results.length,
                total: Math.ceil(data.length / this.#config.batchSize)
            });
        }
        
        return results;
    }
    
    // Generator method
    * #createBatches(data) {
        for (let i = 0; i < data.length; i += this.#config.batchSize) {
            yield data.slice(i, i + this.#config.batchSize);
        }
    }
    
    // Async generator
    async * streamResults(query) {
        let page = 1;
        let hasMore = true;
        
        while (hasMore) {
            try {
                const response = await this.#fetchPage(query, page);
                
                for (const item of response.data) {
                    yield await this.#processItem(item);
                }
                
                hasMore = response.hasMore;
                page++;
                
                // Rate limiting
                await this.#delay(100);
            } catch (error) {
                console.error(`Error processing page ${page}:`, error);
                break;
            }
        }
    }
    
    // Method with destructuring and rest parameters
    async #processBatch(batch, ...options) {
        const [retryCount = 0, timeout = this.#config.timeout] = options;
        
        try {
            // Using Promise.allSettled for parallel processing
            const promises = batch.map(async (item, index) => {
                const cacheKey = this.#getCacheKey(item);
                
                if (this.#cache.has(cacheKey)) {
                    return { item, result: this.#cache.get(cacheKey), cached: true };
                }
                
                const result = await this.#processItem(item, { timeout, index });
                this.#cache.set(cacheKey, result);
                
                return { item, result, cached: false };
            });
            
            const results = await Promise.allSettled(promises);
            
            return results.map((result, index) => ({
                index,
                status: result.status,
                value: result.status === 'fulfilled' ? result.value : null,
                error: result.status === 'rejected' ? result.reason : null
            }));
            
        } catch (error) {
            if (retryCount < DataProcessor.MAX_RETRIES) {
                console.warn(`Batch processing failed, retrying... (${retryCount + 1}/${DataProcessor.MAX_RETRIES})`);
                await this.#delay(1000 * Math.pow(2, retryCount)); // Exponential backoff
                return this.#processBatch(batch, retryCount + 1, timeout);
            }
            throw error;
        }
    }
    
    // Private method with complex logic
    async #processItem(item, { timeout = 5000, index = 0 } = {}) {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeout);
        
        try {
            // Complex object destructuring
            const {
                id,
                type = 'default',
                metadata: { priority = 'normal', tags = [] } = {},
                ...rest
            } = item;
            
            // Template literals and computed properties
            const processedItem = {
                id,
                type,
                priority,
                tags: [...tags, 'processed'],
                processedAt: new Date().toISOString(),
                processingIndex: index,
                hash: createHash('md5').update(JSON.stringify(rest)).digest('hex'),
                [SECRET_KEY]: 'internal_data'
            };
            
            // Conditional logic with optional chaining and nullish coalescing
            const apiUrl = item.metadata?.apiEndpoint ?? `${API_BASE_URL}/process`;
            const requestData = {
                ...processedItem,
                options: rest.options || {}
            };
            
            // Fetch with abort signal
            const response = await fetch(apiUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(requestData),
                signal: controller.signal
            });
            
            if (!response.ok) {
                throw new Error(`API request failed: ${response.status} ${response.statusText}`);
            }
            
            const result = await response.json();
            return { ...processedItem, apiResult: result };
            
        } finally {
            clearTimeout(timeoutId);
        }
    }
    
    #getCacheKey(item) {
        return `${item.id}_${item.type}_${JSON.stringify(item.metadata || {})}`;
    }
    
    async #fetchPage(query, page) {
        const url = new URL(`${API_BASE_URL}/search`);
        url.searchParams.set('q', query);
        url.searchParams.set('page', page.toString());
        
        const response = await fetch(url.toString());
        
        if (!response.ok) {
            throw new Error(`Failed to fetch page ${page}: ${response.statusText}`);
        }
        
        return response.json();
    }
    
    #delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
    
    // Static factory method
    static create(config) {
        return new DataProcessor(config);
    }
    
    // Static method with complex logic
    static async validateData(data) {
        const validator = (item) => {
            const requiredFields = ['id', 'type'];
            return requiredFields.every(field => field in item);
        };
        
        const validItems = [];
        const invalidItems = [];
        
        for (const [index, item] of data.entries()) {
            if (validator(item)) {
                validItems.push({ ...item, index });
            } else {
                invalidItems.push({ item, index, reason: 'Missing required fields' });
            }
        }
        
        return { validItems, invalidItems };
    }
}

// Function with advanced features
const createProcessingPipeline = (() => {
    // Closure variables
    let pipelineCount = 0;
    const activePipelines = new WeakMap();
    
    return function(stages, options = {}) {
        pipelineCount++;
        const pipelineId = `pipeline_${pipelineCount}`;
        
        // Arrow function with destructuring
        const pipeline = async (data) => {
            const {
                onProgress = () => {},
                onError = console.error,
                parallel = false
            } = options;
            
            let currentData = data;
            
            for (const [index, stage] of stages.entries()) {
                try {
                    onProgress({ stage: index, total: stages.length, pipelineId });
                    
                    if (parallel && Array.isArray(currentData)) {
                        // Parallel processing using Promise.all
                        currentData = await Promise.all(
                            currentData.map(item => stage(item))
                        );
                    } else {
                        // Sequential processing
                        currentData = await stage(currentData);
                    }
                } catch (error) {
                    onError(error, { stage: index, pipelineId });
                    throw error;
                }
            }
            
            return currentData;
        };
        
        // Store pipeline metadata
        activePipelines.set(pipeline, {
            id: pipelineId,
            stages: stages.length,
            created: Date.now()
        });
        
        return pipeline;
    };
})();

// React component using hooks and modern patterns
const DataVisualization = ({ data, onUpdate }) => {
    const [processedData, setProcessedData] = React.useState([]);
    const [loading, setLoading] = React.useState(false);
    const [error, setError] = React.useState(null);
    
    // Custom hook
    const useDataProcessor = () => {
        const processor = React.useMemo(() => 
            DataProcessor.create({ batchSize: 50 }), []
        );
        
        React.useEffect(() => {
            const handleProgress = (event) => {
                console.log('Processing progress:', event);
            };
            
            processor.on('processing:progress', handleProgress);
            
            return () => {
                processor.off('processing:progress', handleProgress);
            };
        }, [processor]);
        
        return processor;
    };
    
    const processor = useDataProcessor();
    
    // Effect with cleanup
    React.useEffect(() => {
        let cancelled = false;
        
        const processData = async () => {
            if (!data || data.length === 0) return;
            
            setLoading(true);
            setError(null);
            
            try {
                const results = await processor.processData(data);
                
                if (!cancelled) {
                    setProcessedData(results);
                    onUpdate?.(results);
                }
            } catch (err) {
                if (!cancelled) {
                    setError(err.message);
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        };
        
        processData();
        
        return () => {
            cancelled = true;
        };
    }, [data, processor, onUpdate]);
    
    // Conditional rendering with logical operators
    return loading ? (
        <div>Processing data...</div>
    ) : error ? (
        <div>Error: {error}</div>
    ) : (
        <div>
            <h3>Processed Data ({processedData.length} items)</h3>
            {processedData.map((item, index) => (
                <div key={item.id || index}>
                    {JSON.stringify(item, null, 2)}
                </div>
            ))}
        </div>
    );
};

// Export statements
export default DataProcessor;
export { createProcessingPipeline, Status, DataVisualization };

// Complex export with renaming
export { 
    DataProcessor as AdvancedProcessor,
    API_BASE_URL as DefaultApiUrl 
};

// Re-export from other modules
export { EventEmitter } from 'events';

// Usage example
async function demonstrateFeatures() {
    const processor = DataProcessor.create({
        batchSize: 25,
        retries: 5
    });
    
    const sampleData = Array.from({ length: 100 }, (_, i) => ({
        id: `item_${i}`,
        type: i % 2 === 0 ? 'even' : 'odd',
        metadata: {
            priority: Math.random() > 0.5 ? 'high' : 'normal',
            tags: [`tag_${i % 5}`, 'sample']
        },
        value: Math.random() * 1000
    }));
    
    // Validation
    const { validItems, invalidItems } = await DataProcessor.validateData(sampleData);
    console.log(`Valid: ${validItems.length}, Invalid: ${invalidItems.length}`);
    
    // Processing pipeline
    const pipeline = createProcessingPipeline([
        (data) => data.filter(item => item.type === 'even'),
        (data) => data.map(item => ({ ...item, processed: true })),
        (data) => data.sort((a, b) => b.value - a.value)
    ], { parallel: true });
    
    const pipelineResult = await pipeline(validItems);
    console.log('Pipeline result:', pipelineResult.length, 'items');
    
    // Stream processing
    console.log('Streaming results:');
    for await (const result of processor.streamResults('test_query')) {
        console.log('Streamed:', result);
        // Process only first 5 results for demo
        if (result.processingIndex >= 4) break;
    }
    
    return pipelineResult;
}

// Self-executing async function
(async () => {
    try {
        const results = await demonstrateFeatures();
        console.log('Demo completed with', results.length, 'results');
    } catch (error) {
        console.error('Demo failed:', error);
    }
})();