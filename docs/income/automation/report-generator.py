#!/usr/bin/env python3
"""
Automated Report Generator for Code Reviews
"""
import sys
import json
import yaml
from datetime import datetime
from pathlib import Path
from jinja2 import Environment, FileSystemLoader

class ReportGenerator:
    def __init__(self, template_dir="templates"):
        self.env = Environment(loader=FileSystemLoader(template_dir))
        self.template = self.env.get_template("code_review_template.md")
        
    def load_analysis_data(self, analysis_file):
        """Load analysis results from GitHub Actions"""
        with open(analysis_file) as f:
            return json.load(f)
    
    def load_security_data(self, security_file):
        """Load security scan results"""
        with open(security_file) as f:
            return json.load(f)
    
    def load_performance_data(self, performance_file):
        """Load performance analysis results"""
        with open(performance_file) as f:
            return json.load(f)
    
    def generate_report(self, project_name, analysis_data, security_data, performance_data):
        """Generate the final report"""
        report_data = {
            "project_name": project_name,
            "date": datetime.now().strftime("%Y-%m-%d"),
            "reviewer": "DataScienceBioLab",
            "analysis": analysis_data,
            "security": security_data,
            "performance": performance_data,
            "metrics": self._calculate_metrics(analysis_data),
        }
        
        return self.template.render(**report_data)
    
    def _calculate_metrics(self, analysis_data):
        """Calculate report metrics"""
        return {
            "files_reviewed": len(analysis_data.get("files", [])),
            "critical_issues": len(analysis_data.get("critical", [])),
            "major_issues": len(analysis_data.get("major", [])),
            "minor_issues": len(analysis_data.get("minor", [])),
            "best_practices": len(analysis_data.get("best_practices", [])),
        }
    
    def save_report(self, report_content, output_file):
        """Save the generated report"""
        with open(output_file, 'w') as f:
            f.write(report_content)

def main():
    if len(sys.argv) < 5:
        print("Usage: report-generator.py <project_name> <analysis_file> <security_file> <performance_file>")
        sys.exit(1)
    
    project_name = sys.argv[1]
    analysis_file = sys.argv[2]
    security_file = sys.argv[3]
    performance_file = sys.argv[4]
    
    generator = ReportGenerator()
    
    # Load data
    analysis_data = generator.load_analysis_data(analysis_file)
    security_data = generator.load_security_data(security_file)
    performance_data = generator.load_performance_data(performance_file)
    
    # Generate report
    report = generator.generate_report(
        project_name,
        analysis_data,
        security_data,
        performance_data
    )
    
    # Save report
    output_file = f"reports/{project_name}-review-{datetime.now().strftime('%Y%m%d')}.md"
    generator.save_report(report, output_file)
    print(f"Report generated: {output_file}")

if __name__ == "__main__":
    main() 