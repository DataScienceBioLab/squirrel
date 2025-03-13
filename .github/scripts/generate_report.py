#!/usr/bin/env python3

import json
import os
import sys
from datetime import datetime
from pathlib import Path

class ReportGenerator:
    def __init__(self):
        self.report_dir = Path("reports")
        self.report_dir.mkdir(exist_ok=True)
        self.report_path = self.report_dir / "analysis-report.md"
        
    def generate_report(self):
        """Generate the analysis report."""
        report_content = []
        
        # Add header
        report_content.extend([
            "# Code Analysis Report",
            f"Generated on: {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC')}",
            "",
            "## Summary",
            "",
        ])
        
        # Add clippy results
        clippy_results = self.process_clippy_output()
        if clippy_results:
            report_content.extend([
                "### Rust Analysis",
                "",
                "#### Clippy Findings",
                "",
                *clippy_results,
                "",
            ])
        
        # Add formatting check results
        fmt_results = self.process_fmt_output()
        if fmt_results:
            report_content.extend([
                "#### Formatting Issues",
                "",
                *fmt_results,
                "",
            ])
        
        # Add test results
        test_results = self.process_test_output()
        if test_results:
            report_content.extend([
                "### Test Results",
                "",
                *test_results,
                "",
            ])
        
        # Write report
        self.report_path.write_text("\n".join(report_content))
        print(f"Report generated at {self.report_path}")

    def process_clippy_output(self):
        """Process Clippy output for the report."""
        results = []
        try:
            # Read Clippy output from stderr redirection
            clippy_output = Path("clippy-output.txt")
            if clippy_output.exists():
                warnings = []
                for line in clippy_output.read_text().splitlines():
                    if "warning:" in line:
                        warnings.append(line)
                
                if warnings:
                    results.extend([
                        f"Found {len(warnings)} clippy warnings:",
                        "",
                        "```",
                        *warnings,
                        "```",
                    ])
                else:
                    results.append("No clippy warnings found.")
        except Exception as e:
            results.append(f"Error processing clippy output: {e}")
        
        return results

    def process_fmt_output(self):
        """Process rustfmt output for the report."""
        results = []
        try:
            # Read fmt output from stderr redirection
            fmt_output = Path("fmt-output.txt")
            if fmt_output.exists():
                formatting_issues = []
                for line in fmt_output.read_text().splitlines():
                    if "Diff in" in line:
                        formatting_issues.append(line)
                
                if formatting_issues:
                    results.extend([
                        f"Found {len(formatting_issues)} formatting issues:",
                        "",
                        "```",
                        *formatting_issues,
                        "```",
                    ])
                else:
                    results.append("No formatting issues found.")
        except Exception as e:
            results.append(f"Error processing fmt output: {e}")
        
        return results

    def process_test_output(self):
        """Process test output for the report."""
        results = []
        try:
            # Read test output from stderr redirection
            test_output = Path("test-output.txt")
            if test_output.exists():
                test_content = test_output.read_text()
                if "test result: ok" in test_content:
                    results.append("✅ All tests passed")
                else:
                    failed_tests = []
                    for line in test_content.splitlines():
                        if "test result: FAILED" in line:
                            failed_tests.append(line)
                    
                    if failed_tests:
                        results.extend([
                            "❌ Some tests failed:",
                            "",
                            "```",
                            *failed_tests,
                            "```",
                        ])
        except Exception as e:
            results.append(f"Error processing test output: {e}")
        
        return results

def main():
    generator = ReportGenerator()
    generator.generate_report()

if __name__ == "__main__":
    main() 