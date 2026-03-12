// Package framework provides test reporting capabilities for chaos tests
package framework

import (
	"encoding/json"
	"fmt"
	"html/template"
	"os"
	"path/filepath"
	"time"
)

// Reporter generates test reports in multiple formats
type Reporter struct {
	outputDir string
	format    string
}

// ReportData contains all information needed for report generation
type ReportData struct {
	Timestamp   time.Time
	Results     []TestResult
	Summary     map[string]interface{}
	Duration    time.Duration
	Environment map[string]string
}

// NewReporter creates a new reporter with default settings
func NewReporter() *Reporter {
	return &Reporter{
		outputDir: "./reports",
		format:    "json",
	}
}

// SetOutputDir configures the directory for report output
func (r *Reporter) SetOutputDir(dir string) {
	r.outputDir = dir
}

// SetFormat configures the report format (json, html, text)
func (r *Reporter) SetFormat(format string) {
	r.format = format
}

// GenerateReport creates a comprehensive test report
func (r *Reporter) GenerateReport(results []TestResult) error {
	if err := os.MkdirAll(r.outputDir, 0755); err != nil {
		return fmt.Errorf("failed to create report directory: %w", err)
	}

	data := ReportData{
		Timestamp: time.Now(),
		Results:   results,
		Summary:   r.calculateSummary(results),
		Duration:  r.calculateTotalDuration(results),
		Environment: map[string]string{
			"go_version":  os.Getenv("GO_VERSION"),
			"platform":    os.Getenv("GOOS") + "/" + os.Getenv("GOARCH"),
			"test_run_id": fmt.Sprintf("chaos-%d", time.Now().Unix()),
		},
	}

	// Generate JSON report
	if err := r.generateJSONReport(data); err != nil {
		return fmt.Errorf("failed to generate JSON report: %w", err)
	}

	// Generate HTML report
	if err := r.generateHTMLReport(data); err != nil {
		return fmt.Errorf("failed to generate HTML report: %w", err)
	}

	// Generate text summary
	if err := r.generateTextReport(data); err != nil {
		return fmt.Errorf("failed to generate text report: %w", err)
	}

	return nil
}

// calculateSummary computes test statistics
func (r *Reporter) calculateSummary(results []TestResult) map[string]interface{} {
	passed := 0
	failed := 0
	totalDuration := time.Duration(0)

	for _, result := range results {
		if result.Passed {
			passed++
		} else {
			failed++
		}
		totalDuration += result.Duration
	}

	successRate := 0.0
	if len(results) > 0 {
		successRate = float64(passed) / float64(len(results)) * 100
	}

	return map[string]interface{}{
		"total_tests":    len(results),
		"passed":         passed,
		"failed":         failed,
		"success_rate":   fmt.Sprintf("%.2f%%", successRate),
		"total_duration": totalDuration.String(),
		"avg_duration":   (totalDuration / time.Duration(len(results)+1)).String(),
	}
}

// calculateTotalDuration sums up all test durations
func (r *Reporter) calculateTotalDuration(results []TestResult) time.Duration {
	var total time.Duration
	for _, result := range results {
		total += result.Duration
	}
	return total
}

// generateJSONReport creates a machine-readable JSON report
func (r *Reporter) generateJSONReport(data ReportData) error {
	filename := filepath.Join(r.outputDir, fmt.Sprintf("chaos-report-%d.json", data.Timestamp.Unix()))

	file, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create JSON report file: %w", err)
	}
	defer file.Close()

	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")

	if err := encoder.Encode(data); err != nil {
		return fmt.Errorf("failed to encode JSON report: %w", err)
	}

	fmt.Printf("[Reporter] JSON report generated: %s\n", filename)
	return nil
}

// generateHTMLReport creates a human-readable HTML report
func (r *Reporter) generateHTMLReport(data ReportData) error {
	const htmlTemplate = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SMA-OS Chaos Test Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            border-radius: 8px;
            margin-bottom: 30px;
        }
        .header h1 {
            margin: 0 0 10px 0;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .summary-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .summary-card h3 {
            margin: 0 0 10px 0;
            color: #666;
            font-size: 14px;
            text-transform: uppercase;
        }
        .summary-card .value {
            font-size: 32px;
            font-weight: bold;
            color: #333;
        }
        .summary-card.passed .value { color: #28a745; }
        .summary-card.failed .value { color: #dc3545; }
        .results-table {
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        table {
            width: 100%;
            border-collapse: collapse;
        }
        th {
            background: #f8f9fa;
            padding: 15px;
            text-align: left;
            font-weight: 600;
            color: #666;
            border-bottom: 2px solid #dee2e6;
        }
        td {
            padding: 15px;
            border-bottom: 1px solid #dee2e6;
        }
        tr:hover {
            background: #f8f9fa;
        }
        .status {
            display: inline-block;
            padding: 5px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
        }
        .status.passed {
            background: #d4edda;
            color: #155724;
        }
        .status.failed {
            background: #f8d7da;
            color: #721c24;
        }
        .error-message {
            color: #dc3545;
            font-size: 12px;
            margin-top: 5px;
        }
        .timestamp {
            color: #666;
            font-size: 12px;
        }
        .footer {
            text-align: center;
            margin-top: 30px;
            color: #666;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>SMA-OS Chaos Test Report</h1>
        <p>Generated: {{.Timestamp.Format "2006-01-02 15:04:05"}}</p>
    </div>

    <div class="summary">
        <div class="summary-card">
            <h3>Total Tests</h3>
            <div class="value">{{.Summary.total_tests}}</div>
        </div>
        <div class="summary-card passed">
            <h3>Passed</h3>
            <div class="value">{{.Summary.passed}}</div>
        </div>
        <div class="summary-card failed">
            <h3>Failed</h3>
            <div class="value">{{.Summary.failed}}</div>
        </div>
        <div class="summary-card">
            <h3>Success Rate</h3>
            <div class="value">{{.Summary.success_rate}}</div>
        </div>
    </div>

    <div class="results-table">
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Status</th>
                    <th>Duration</th>
                    <th>Timestamp</th>
                    <th>Error</th>
                </tr>
            </thead>
            <tbody>
                {{range .Results}}
                <tr>
                    <td><strong>{{.Name}}</strong></td>
                    <td>
                        {{if .Passed}}
                        <span class="status passed">Passed</span>
                        {{else}}
                        <span class="status failed">Failed</span>
                        {{end}}
                    </td>
                    <td>{{.Duration}}</td>
                    <td class="timestamp">{{.Timestamp.Format "15:04:05"}}</td>
                    <td>
                        {{if .Error}}
                        <div class="error-message">{{.Error}}</div>
                        {{else}}-
                        {{end}}
                    </td>
                </tr>
                {{end}}
            </tbody>
        </table>
    </div>

    <div class="footer">
        <p>SMA-OS Chaos Engineering Framework | Test Run ID: {{.Environment.test_run_id}}</p>
    </div>
</body>
</html>`

	filename := filepath.Join(r.outputDir, fmt.Sprintf("chaos-report-%d.html", data.Timestamp.Unix()))

	file, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create HTML report file: %w", err)
	}
	defer file.Close()

	tmpl, err := template.New("report").Parse(htmlTemplate)
	if err != nil {
		return fmt.Errorf("failed to parse HTML template: %w", err)
	}

	if err := tmpl.Execute(file, data); err != nil {
		return fmt.Errorf("failed to execute HTML template: %w", err)
	}

	fmt.Printf("[Reporter] HTML report generated: %s\n", filename)
	return nil
}

// generateTextReport creates a simple text summary
func (r *Reporter) generateTextReport(data ReportData) error {
	filename := filepath.Join(r.outputDir, fmt.Sprintf("chaos-report-%d.txt", data.Timestamp.Unix()))

	file, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create text report file: %w", err)
	}
	defer file.Close()

	fmt.Fprintf(file, "SMA-OS Chaos Test Report\n")
	fmt.Fprintf(file, "========================\n\n")
	fmt.Fprintf(file, "Generated: %s\n", data.Timestamp.Format("2006-01-02 15:04:05"))
	fmt.Fprintf(file, "Test Run ID: %s\n\n", data.Environment["test_run_id"])

	fmt.Fprintf(file, "Summary:\n")
	fmt.Fprintf(file, "--------\n")
	for key, value := range data.Summary {
		fmt.Fprintf(file, "  %s: %v\n", key, value)
	}
	fmt.Fprintf(file, "\n")

	fmt.Fprintf(file, "Results:\n")
	fmt.Fprintf(file, "--------\n")
	for _, result := range data.Results {
		status := "PASS"
		if !result.Passed {
			status = "FAIL"
		}
		fmt.Fprintf(file, "[%s] %s (%s)\n", status, result.Name, result.Duration)
		if result.Error != nil {
			fmt.Fprintf(file, "  Error: %s\n", result.Error.Error())
		}
	}

	fmt.Printf("[Reporter] Text report generated: %s\n", filename)
	return nil
}

// PrintSummary outputs a summary to stdout
func (r *Reporter) PrintSummary(results []TestResult) {
	summary := r.calculateSummary(results)

	fmt.Println("\n========================================")
	fmt.Println("      Chaos Test Summary")
	fmt.Println("========================================")
	fmt.Printf("Total Tests:  %v\n", summary["total_tests"])
	fmt.Printf("Passed:       %v\n", summary["passed"])
	fmt.Printf("Failed:       %v\n", summary["failed"])
	fmt.Printf("Success Rate: %v\n", summary["success_rate"])
	fmt.Printf("Duration:     %v\n", summary["total_duration"])
	fmt.Println("========================================\n")
}
