"""
This script automates the process of validating an OGC API - Processes implementation
using the OGC CITE validator. It performs the following steps:

1. Loads environment variables from a `.env` file.
2. Starts the OGC API - Processes server using a specified database URL.
3. Starts the OGC CITE validator in a containerized environment.
4. Runs the validation tests by invoking the validator's REST API.
5. Parses the test results from the generated XML file.
6. Outputs the number of tests skipped, succeeded, and failed.
7. Exits with a non-zero status code if any tests fail.

Dependencies:
- Python standard libraries: `time`, `socket`, `subprocess`, `sys`, `os`, `xml.etree.ElementTree`.
- Third-party libraries: `dotenv`, `requests`.

Environment Variables:
- `DATABASE_URL`: The database connection string required by the OGC API - Processes server.

Usage:
Run the script directly to execute the validation process. Ensure that the required
dependencies are installed and the `DATABASE_URL` environment variable is set.

Note:
- The script assumes that `cargo` is installed and configured for running the OGC API server.
- The script uses `podman` to run the OGC CITE validator container.
- The test results are saved to a file named `results.xml` in the current working directory.
"""

from collections.abc import Generator
import time
import socket
import subprocess
import sys
import os
import xml.etree.ElementTree as ET
from dotenv import load_dotenv
import requests

def eprint(*args, **kwargs):
    """Prints to stderr."""
    print(*args, file=sys.stderr, **kwargs)

def wait_for_port(host, port):
    """Waits until a TCP port is open on the specified host."""

    while True:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(1)
            if sock.connect_ex((host, port)) == 0:
                break
        time.sleep(1)

def extract_and_format_results(root: ET.Element) -> Generator[str]:
    """Extracts TestNG results and formats a human-readable report."""

    # Find all test methods
    test_methods = root.findall('.//test-method')

    yield "--- Detailed Failures & Skips ---"

    # Iterate through each test-method element
    for method in test_methods:
        status = method.get('status')
        name = method.get('name')
        class_name = method.get('class')

        # Only report failures and skips for a focused output
        if status in ['FAIL', 'SKIP']:

            # Identify the failure type
            if status == 'FAIL':
                symbol = '❌'
                title = f"Failure in {name} ({class_name}):"
            else:
                symbol = '⚠️'
                title = f"Skipped Test {name} ({class_name}):"

            yield f"\n{symbol} {title}"

            # Extract Exception Message
            exception = method.find('exception')
            if exception is not None:
                message = exception_message(exception)

                # Clean up and indent the message
                indented_message = "\n".join(
                    [f"    -> {line}" for line in message.split('\n')]
                ).strip()
                yield f"    Reason: {indented_message}"
            else:
                yield "    Reason: No specific exception recorded."

    # Get overall counts from the root element
    total = root.get('total')
    passed = root.get('passed')
    failed = root.get('failed')
    skipped = root.get('skipped')

    header = "--- Overall Test Summary ---"
    yield '\n' + header
    yield f"Total Tests: {total}"
    yield f"✅ Passed:   {passed}"
    yield f"❌ Failed:   {failed}"
    yield f"⚠️ Skipped:  {skipped}"
    yield "-" * len(header)

def exception_message(exception: ET.Element) -> str:
    """Extracts and formats the exception message from a test-method element."""
    message = exception.find('message')

    if message is None or message.text is None:
        return "No message provided."

    return message.text.strip()

def main():
    """Main function to run the validation process."""

    ogcapi_server = None
    validator_server = None

    try:
        load_dotenv()
        database_url = os.getenv('DATABASE_URL')
        if not database_url:
            eprint("DATABASE_URL environment variable is not set.")
            exit(1)

        eprint("Starting OGC API - Processes server…")
        ogcapi_server = subprocess.Popen(
            ['cargo', 'run', '-p', 'demo-service', '--', '--database-url', database_url],
            stderr=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
        )
        wait_for_port('localhost', 8484)

        eprint("Starting OGC CITE validator…")
        validator_server = subprocess.Popen(
            ['podman', 'run', '--network', 'host', 'docker.io/ogccite/ets-ogcapi-processes10'],
            stderr=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
        )
        wait_for_port('localhost', 8080)

        eprint("Running tests…")
        response = requests.get(
            'http://localhost:8080/teamengine/rest/suites/ogcapi-processes-1.0/run',
            auth=('ogctest', 'ogctest'),
            params={
                'iut': 'http://localhost:8484',
                'echoprocessid': 'echo'
            },
            headers={'Accept': 'application/xml'}, # delivers TestNG XML
            timeout=None
        )
        with open('results.xml', 'wb') as f:
            f.write(response.content)

        # Parse the results.xml file
        tree = ET.parse('results.xml')

        for line in extract_and_format_results(tree.getroot()):
            eprint(line)

        # Exit with a non-zero status if any tests failed
        tests_failed = tree.getroot().get('failed')
        if int(tests_failed) != 0:
            exit(1)

    finally:
        # Kill the server processes
        if ogcapi_server:
            ogcapi_server.terminate()
        if validator_server:
            validator_server.terminate()

if __name__ == "__main__":
    main()
