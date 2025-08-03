"""
Complex Python example demonstrating decorators, async/await, metaclasses,
and modern Python features for comprehensive parser testing.
"""

import asyncio
import functools
import time
from typing import List, Dict, Optional, Generic, TypeVar, Protocol
from dataclasses import dataclass, field
from abc import ABC, abstractmethod
from contextlib import asynccontextmanager
from enum import Enum, auto


T = TypeVar('T')
K = TypeVar('K')
V = TypeVar('V')


class Status(Enum):
    PENDING = auto()
    PROCESSING = auto()
    COMPLETED = auto()
    FAILED = auto()


# Custom decorators
def timing_decorator(func):
    """Decorator to measure execution time"""
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        start_time = time.time()
        result = func(*args, **kwargs)
        end_time = time.time()
        print(f"{func.__name__} took {end_time - start_time:.4f} seconds")
        return result
    return wrapper


def async_retry(max_attempts: int = 3, delay: float = 1.0):
    """Async decorator with parameters for retry logic"""
    def decorator(func):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            for attempt in range(max_attempts):
                try:
                    return await func(*args, **kwargs)
                except Exception as e:
                    if attempt == max_attempts - 1:
                        raise
                    print(f"Attempt {attempt + 1} failed: {e}")
                    await asyncio.sleep(delay)
            return None
        return wrapper
    return decorator


class Singleton(type):
    """Metaclass implementing singleton pattern"""
    _instances = {}
    
    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]


# Protocol for type hints
class Drawable(Protocol):
    def draw(self) -> str:
        ...


@dataclass
class Point:
    x: float
    y: float
    z: float = 0.0
    metadata: Dict[str, str] = field(default_factory=dict)
    
    def distance_to(self, other: 'Point') -> float:
        return ((self.x - other.x)**2 + (self.y - other.y)**2 + (self.z - other.z)**2)**0.5


class DatabaseManager(metaclass=Singleton):
    """Singleton database manager with async operations"""
    
    def __init__(self):
        self.connections: Dict[str, str] = {}
        self.status = Status.PENDING
    
    @async_retry(max_attempts=5, delay=0.5)
    async def connect(self, connection_string: str) -> bool:
        """Async method with retry decorator"""
        print(f"Connecting to: {connection_string}")
        # Simulate connection attempt
        await asyncio.sleep(0.1)
        if len(connection_string) < 10:  # Simulate failure condition
            raise ConnectionError("Invalid connection string")
        
        self.connections[connection_string] = "active"
        self.status = Status.COMPLETED
        return True
    
    @timing_decorator
    def get_connection_count(self) -> int:
        """Method with timing decorator"""
        return len(self.connections)


class Shape(ABC):
    """Abstract base class with generic type support"""
    
    def __init__(self, name: str):
        self.name = name
    
    @abstractmethod
    def area(self) -> float:
        pass
    
    @abstractmethod
    def perimeter(self) -> float:
        pass
    
    def __str__(self) -> str:
        return f"{self.name}: area={self.area():.2f}, perimeter={self.perimeter():.2f}"


class Rectangle(Shape):
    def __init__(self, name: str, width: float, height: float):
        super().__init__(name)
        self.width = width
        self.height = height
    
    def area(self) -> float:
        return self.width * self.height
    
    def perimeter(self) -> float:
        return 2 * (self.width + self.height)
    
    def draw(self) -> str:
        return f"Drawing rectangle {self.name}"


class Circle(Shape):
    def __init__(self, name: str, radius: float):
        super().__init__(name)
        self.radius = radius
    
    def area(self) -> float:
        return 3.14159 * self.radius ** 2
    
    def perimeter(self) -> float:
        return 2 * 3.14159 * self.radius
    
    def draw(self) -> str:
        return f"Drawing circle {self.name}"


class Container(Generic[T]):
    """Generic container class"""
    
    def __init__(self):
        self._items: List[T] = []
    
    def add(self, item: T) -> None:
        self._items.append(item)
    
    def get_all(self) -> List[T]:
        return self._items.copy()
    
    def filter(self, predicate) -> List[T]:
        return [item for item in self._items if predicate(item)]
    
    # Generator method
    def iterate(self):
        for item in self._items:
            yield item


class AsyncProcessor:
    """Class demonstrating async context managers and generators"""
    
    def __init__(self, batch_size: int = 10):
        self.batch_size = batch_size
        self.processed_count = 0
    
    @asynccontextmanager
    async def processing_session(self):
        """Async context manager"""
        print("Starting processing session")
        try:
            yield self
        finally:
            print(f"Processed {self.processed_count} items")
            self.processed_count = 0
    
    async def process_items(self, items: List[str]) -> List[str]:
        """Async generator usage"""
        results = []
        async for batch in self._batch_items(items):
            processed_batch = await self._process_batch(batch)
            results.extend(processed_batch)
        return results
    
    async def _batch_items(self, items: List[str]):
        """Async generator"""
        for i in range(0, len(items), self.batch_size):
            batch = items[i:i + self.batch_size]
            yield batch
            await asyncio.sleep(0.01)  # Yield control
    
    async def _process_batch(self, batch: List[str]) -> List[str]:
        """Process a batch of items"""
        await asyncio.sleep(0.1)  # Simulate processing
        self.processed_count += len(batch)
        return [f"processed_{item}" for item in batch]


# Complex function with multiple decorators and type hints
@timing_decorator
@async_retry(max_attempts=3)
async def complex_calculation(
    data: Dict[str, List[int]], 
    multiplier: float = 1.0,
    callback: Optional[callable] = None
) -> Dict[str, float]:
    """
    Complex async function with comprehensive type hints
    """
    results = {}
    
    for key, values in data.items():
        # List comprehension
        processed_values = [v * multiplier for v in values if v > 0]
        
        # Dictionary comprehension  
        squared = {f"{key}_{i}": v**2 for i, v in enumerate(processed_values)}
        
        # Generator expression
        result = sum(v for v in squared.values())
        results[key] = result
        
        if callback:
            await callback(key, result)
    
    return results


# Usage demonstration with advanced features
async def demonstrate_features():
    """Demonstrate all the complex features"""
    
    # Singleton pattern
    db1 = DatabaseManager()
    db2 = DatabaseManager()
    assert db1 is db2, "Singleton not working"
    
    # Generic containers
    shape_container: Container[Shape] = Container()
    shape_container.add(Rectangle("rect1", 10, 5))
    shape_container.add(Circle("circle1", 3))
    
    # Protocol usage
    drawable_shapes: List[Drawable] = [Rectangle("rect2", 8, 4), Circle("circle2", 2)]
    
    # Async context manager
    processor = AsyncProcessor(batch_size=5)
    async with processor.processing_session():
        items = [f"item_{i}" for i in range(20)]
        results = await processor.process_items(items)
    
    # Complex function usage
    test_data = {
        "series_a": [1, 2, 3, 4, 5],
        "series_b": [10, 20, 30, -5, 15],
        "series_c": [100, -50, 75, 25]
    }
    
    async def progress_callback(key: str, result: float):
        print(f"Processed {key}: {result}")
    
    calculation_results = await complex_calculation(
        test_data, 
        multiplier=2.0, 
        callback=progress_callback
    )
    
    return calculation_results


if __name__ == "__main__":
    # Run the demonstration
    results = asyncio.run(demonstrate_features())
    print(f"Final results: {results}")
    
    # Dataclass usage
    p1 = Point(1.0, 2.0, 3.0, {"color": "red"})
    p2 = Point(4.0, 5.0, 6.0)
    distance = p1.distance_to(p2)
    print(f"Distance between points: {distance:.2f}")