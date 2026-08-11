# Code Examples

This chapter tests syntax highlighting and code block rendering.

## Rust Code

```rust
fn main() {
    println!("Hello, world!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
    
    for num in doubled {
        println!("Doubled: {}", num);
    }
}

#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}
```

## JavaScript Code

```javascript
// Modern JavaScript with ES6+ features
class Calculator {
    constructor() {
        this.result = 0;
    }
    
    add(num) {
        this.result += num;
        return this;
    }
    
    multiply(num) {
        this.result *= num;
        return this;
    }
    
    getValue() {
        return this.result;
    }
}

const calc = new Calculator();
const result = calc.add(5).multiply(3).getValue();
console.log(`Result: ${result}`);

// Arrow functions and async/await
const fetchData = async (url) => {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Error fetching data:', error);
        throw error;
    }
};
```

## Python Code

```python
import asyncio
from typing import List, Optional
from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int
    email: Optional[str] = None
    
    def __post_init__(self):
        if self.age < 0:
            raise ValueError("Age cannot be negative")
    
    def is_adult(self) -> bool:
        return self.age >= 18

class PersonManager:
    def __init__(self):
        self.people: List[Person] = []
    
    def add_person(self, person: Person):
        self.people.append(person)
    
    def find_adults(self) -> List[Person]:
        return [p for p in self.people if p.is_adult()]
    
    async def send_emails(self):
        tasks = []
        for person in self.people:
            if person.email:
                tasks.append(self.send_email(person))
        
        await asyncio.gather(*tasks)
    
    async def send_email(self, person: Person):
        # Simulate email sending
        await asyncio.sleep(0.1)
        print(f"Email sent to {person.name} at {person.email}")

# Example usage
async def main():
    manager = PersonManager()
    manager.add_person(Person("Alice", 30, "alice@example.com"))
    manager.add_person(Person("Bob", 17, "bob@example.com"))
    
    adults = manager.find_adults()
    print(f"Found {len(adults)} adults")
    
    await manager.send_emails()

if __name__ == "__main__":
    asyncio.run(main())
```

## HTML/CSS Code

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Page</title>
    <style>
        .container {
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        
        .card {
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
            padding: 20px;
            margin-bottom: 20px;
        }
        
        .card h2 {
            color: #333;
            margin-top: 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="card">
            <h2>Welcome</h2>
            <p>This is a sample HTML page with embedded CSS.</p>
        </div>
    </div>
</body>
</html>
```

## SQL Code

```sql
-- Database schema for a blog system
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    content TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id),
    published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_posts_author ON posts(author_id);
CREATE INDEX idx_posts_published ON posts(published);
CREATE INDEX idx_posts_created ON posts(created_at);

-- Complex query with JOIN and aggregation
SELECT 
    u.username,
    COUNT(p.id) as post_count,
    MAX(p.created_at) as latest_post
FROM users u
LEFT JOIN posts p ON u.id = p.author_id AND p.published = TRUE
GROUP BY u.id, u.username
HAVING COUNT(p.id) > 0
ORDER BY post_count DESC, latest_post DESC;
```

## Plain Code Block

```
This is a plain code block without syntax highlighting.
It should still be rendered in a monospace font
and preserve whitespace and indentation.

    Indented text
        More indented text
            Even more indented

Special characters: <>&"'
```

## Inline Code

Here's some inline code: `const greeting = "Hello, world!";`

And here's a longer inline code snippet: `fetch('/api/data').then(response => response.json()).then(data => console.log(data));`

Previous: [Basic Features](basic.md) | Next: [Advanced Features](advanced.md)