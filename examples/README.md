# PPM Examples

This directory contains example projects demonstrating different use cases for PPM.

## Available Examples

### 1. [Fullstack Web Application](./fullstack-webapp/)
A complete web application with React frontend and Flask backend.
- **Technologies**: React, TypeScript, Flask, SQLAlchemy
- **Use Case**: Full-stack development with modern web technologies
- **Features**: API server, database integration, frontend build pipeline

### 2. [Data Science Project](./data-science/)
A data analysis project using Python with Jupyter notebook support.
- **Technologies**: Pandas, NumPy, Matplotlib, Jupyter
- **Use Case**: Data analysis and visualization
- **Features**: Notebook environment, data processing, visualization

### 3. [Node.js with Python Scripts](./node-python-hybrid/)
A Node.js application that uses Python scripts for data processing.
- **Technologies**: Express.js, Python scripts, pandas
- **Use Case**: Hybrid applications leveraging both ecosystems
- **Features**: API endpoints, background processing, data transformation

### 4. [CLI Tool Development](./cli-tool/)
Development environment for building command-line tools.
- **Technologies**: Node.js, Python, testing frameworks
- **Use Case**: CLI tool development and testing
- **Features**: Cross-platform scripts, automated testing

### 5. [Microservices](./microservices/)
Multiple services in different languages within a monorepo.
- **Technologies**: Multiple Node.js and Python services
- **Use Case**: Microservices architecture
- **Features**: Service orchestration, shared dependencies

## Running Examples

Each example includes its own README with specific instructions. Generally:

```bash
# Navigate to example directory
cd examples/fullstack-webapp

# Install dependencies
ppm install

# Run the application
ppm run start
```

## Example Structure

Each example follows this structure:
```
example-name/
├── README.md           # Specific instructions for this example
├── project.toml        # PPM configuration
├── src/               # Source code
├── tests/             # Tests
└── docs/              # Additional documentation
```
