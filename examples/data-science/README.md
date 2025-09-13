# Data Science Project Example

A comprehensive data science project demonstrating how to use PPM for Python-focused data analysis and visualization workflows.

## Features

- **Data Processing**: Pandas, NumPy for data manipulation
- **Visualization**: Matplotlib, Seaborn, Plotly for charts and graphs
- **Machine Learning**: Scikit-learn for modeling
- **Jupyter Integration**: Interactive notebook environment
- **Documentation**: Automated report generation

## Project Structure

```
data-science/
├── notebooks/          # Jupyter notebooks for analysis
│   ├── 01-data-exploration.ipynb
│   ├── 02-data-cleaning.ipynb
│   ├── 03-feature-engineering.ipynb
│   └── 04-modeling.ipynb
├── src/               # Reusable Python modules
│   ├── data/          # Data loading and processing
│   ├── features/      # Feature engineering
│   ├── models/        # ML models
│   └── visualization/ # Plotting utilities
├── data/              # Raw and processed datasets
│   ├── raw/
│   ├── processed/
│   └── external/
├── reports/           # Generated reports and figures
├── tests/             # Unit tests for src modules
├── project.toml       # PPM configuration
└── README.md
```

## Getting Started

### 1. Install Dependencies

```bash
cd data-science

# Install all Python dependencies and create virtual environment
ppm install --dev

# Create virtual environment if not done automatically
ppm venv create --python python3.11
```

### 2. Download Sample Data

```bash
# Download sample datasets
ppm run data:download

# Or manually place your data in data/raw/
```

### 3. Start Jupyter

```bash
# Launch Jupyter Lab
ppm run jupyter

# Or start Jupyter Notebook
ppm run notebook
```

### 4. Run Analysis Pipeline

```bash
# Run complete analysis pipeline
ppm run pipeline

# Or run individual steps
ppm run data:clean
ppm run features:generate
ppm run models:train
ppm run reports:generate
```

## Available Scripts

| Script | Description |
|--------|-------------|
| `jupyter` | Start Jupyter Lab server |
| `notebook` | Start Jupyter Notebook server |
| `pipeline` | Run complete analysis pipeline |
| `data:download` | Download sample datasets |
| `data:clean` | Clean and preprocess raw data |
| `features:generate` | Generate features for modeling |
| `models:train` | Train machine learning models |
| `models:evaluate` | Evaluate model performance |
| `reports:generate` | Generate analysis reports |
| `test` | Run unit tests |
| `lint` | Check code quality |
| `format` | Format code with Black |

## Dependencies

### Core Data Science Stack
- **pandas** - Data manipulation and analysis
- **numpy** - Numerical computing
- **matplotlib** - Basic plotting
- **seaborn** - Statistical visualization
- **plotly** - Interactive visualizations
- **scipy** - Scientific computing
- **scikit-learn** - Machine learning

### Jupyter Environment
- **jupyter** - Jupyter notebook server
- **jupyterlab** - Modern notebook interface
- **ipykernel** - Python kernel for Jupyter
- **ipywidgets** - Interactive widgets

### Development Tools
- **pytest** - Testing framework
- **black** - Code formatting
- **flake8** - Code linting
- **mypy** - Type checking
- **coverage** - Test coverage

## Notebooks

### 1. Data Exploration (`01-data-exploration.ipynb`)
- Initial data inspection
- Summary statistics
- Missing value analysis
- Data visualization

### 2. Data Cleaning (`02-data-cleaning.ipynb`)
- Handle missing values
- Remove duplicates
- Fix data types
- Outlier detection

### 3. Feature Engineering (`03-feature-engineering.ipynb`)
- Create new features
- Transform existing features
- Feature selection
- Correlation analysis

### 4. Modeling (`04-modeling.ipynb`)
- Train multiple models
- Cross-validation
- Hyperparameter tuning
- Model evaluation

## Usage Examples

### Basic Data Analysis

```python
import pandas as pd
from src.data.loader import load_dataset
from src.visualization.plots import create_correlation_heatmap

# Load data
df = load_dataset('sales_data.csv')

# Basic analysis
print(df.describe())

# Create visualization
create_correlation_heatmap(df, save_path='reports/correlation.png')
```

### Model Training

```python
from src.models.classifier import train_classifier
from src.features.engineering import create_features

# Prepare features
X, y = create_features(df)

# Train model
model = train_classifier(X, y, model_type='random_forest')

# Evaluate
accuracy = model.score(X_test, y_test)
print(f"Model accuracy: {accuracy:.3f}")
```

### Report Generation

```bash
# Generate comprehensive report
ppm run reports:generate

# This creates:
# - reports/analysis_report.html
# - reports/figures/
# - reports/model_performance.json
```

## Configuration

### Environment Variables

Create a `.env` file:

```env
# Data paths
DATA_DIR=./data
RAW_DATA_DIR=./data/raw
PROCESSED_DATA_DIR=./data/processed

# Model settings
MODEL_RANDOM_STATE=42
TEST_SIZE=0.2

# Jupyter settings
JUPYTER_PORT=8888
JUPYTER_TOKEN=your-token-here
```

### Virtual Environment

The project uses Python 3.11 with a virtual environment in `.venv/`:

```bash
# Check virtual environment info
ppm venv info

# Recreate if needed
ppm venv remove
ppm venv create --python python3.11
```

## Best Practices Demonstrated

1. **Reproducible Analysis**: Fixed random seeds, version pinning
2. **Modular Code**: Reusable functions in `src/` modules
3. **Testing**: Unit tests for data processing functions
4. **Documentation**: Well-documented notebooks and code
5. **Version Control**: Proper .gitignore for data science projects
6. **Environment Management**: Isolated Python environment

## Learning Points

This example shows how to:

- Manage Python data science dependencies with PPM
- Structure a data science project for reproducibility
- Use Jupyter notebooks effectively
- Create reusable data processing modules
- Generate automated reports
- Test data science code properly
