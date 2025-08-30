# skop

## Overview
Skop is a widget-based system investigation tool that provides multi-modal feedback through synchronized audio and visual interfaces. Built around an infinite canvas approach, Skop records system activity and enables detailed replay and analysis of investigations.

## Philosophy
After working ten years as a system administrator, Skop addresses common pain points:
- Losing significant results in terminal backscroll
- Difficulty synchronizing outputs across time
- Poor collaboration on investigative experiments
- Command line tools that afford nothing to beginners
- Missing the physical feedback of systems (CPU whir, disk sounds)

## Architecture

### Investigation-Based Workflow
Skop organizes work into investigations - isolated workspaces containing widgets, configurations, and historical data. Each investigation is a self-contained SQLite file that can be easily shared and analyzed.

### Data Storage
- **Main Database** (`~/.skop/main.db`): Application settings and investigation registry
- **Investigation Files** (`investigation_name.skop`): Complete investigation data including metadata, widget configurations, and timestamped raw data streams

### Raw Data Capture
All command outputs and system metrics are captured as raw text lines with microsecond timestamps:
```sql
CREATE TABLE raw_data (
    id INTEGER PRIMARY KEY,
    widget_id INTEGER,
    timestamp INTEGER,
    line_content TEXT,
    line_number INTEGER
);
```

### Widget Persistence
Widget configurations, positions, and states are automatically saved:
```sql
CREATE TABLE widgets (
    id INTEGER PRIMARY KEY,
    widget_type TEXT,
    config_json TEXT,
    position_x REAL, position_y REAL,
    size_x REAL, size_y REAL,
    created_at INTEGER,
    active BOOLEAN DEFAULT 1
);
```

## User Interface

### Investigation Browser
The startup interface displays an infinite canvas with investigation widgets. Each investigation widget shows name, last modified date, and preview thumbnail. Double-clicking opens the investigation workspace.

### Investigation Workspace
- **Infinite Canvas**: Widgets float freely and can be positioned anywhere
- **Widget Producer Menu**: Left sidebar for creating new widgets
- **Auto-execution**: Widgets begin collecting data immediately upon creation
- **Live Updates**: Real-time data display with configurable refresh rates

### Data Replay System
- **Timeline Controls**: Scrub through historical data with playback speed control (0.1x to 10x)
- **Range Selection**: Focus on specific time periods
- **Widget Filtering**: Replay data from specific widgets or widget types
- **Synchronized Playback**: All widgets replay in temporal synchronization

## Widget Types

### System Monitoring
- **CPU Monitor**: Real-time per-core usage with audio sonification (unique frequencies per core)
- **Process Monitor**: Top processes with color-coded usage indicators using jc JSON parsing
- **Network Monitor**: Connection monitoring with state-based color coding
- **System Info**: Hardware information and system activity displays

### Command Execution
- **SSH Commands**: Execute commands locally or remotely with live output streaming
- **JSON Parsing**: Automatic integration with jc tool for robust command output handling

### Information
- **About Widget**: Tool information, philosophy, and current capabilities

## Audio Feedback
CPU monitoring includes sonification where each CPU core generates a unique frequency tone (200Hz + core*50Hz) with volume modulation based on usage (-40dB to -10dB range). This provides ambient awareness of system load.

## Templates and Sharing
- **Default Templates**: Standard investigation layouts for common scenarios
- **Custom Templates**: Create reusable investigation patterns
- **File Sharing**: Investigation files (.skop) contain complete investigation state
- **Portability**: Single-file investigations enable easy collaboration

## Technical Implementation
- **Rust**: Core tool built with Rust for performance and safety
- **eframe/egui**: Cross-platform GUI framework
- **Kira Audio**: Real-time audio synthesis for system sonification  
- **SQLite**: Vendored database for complete portability
- **Tokio**: Async runtime for command execution and data collection
- **JSON Parsing**: Integration with jc tool for command output processing

## Data Philosophy
Skop records everything happening during investigations, enabling complete replay and analysis. Raw data preservation allows for post-hoc analysis techniques not available during live monitoring. The combination of real-time feedback and historical analysis transforms system investigation from reactive troubleshooting to proactive understanding.