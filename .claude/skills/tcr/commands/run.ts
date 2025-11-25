import { findProjectRoot, findTestFiles, determineTarget } from "../lib/utils";
import { runTests, formatTestOutput } from "../lib/test-runner";
import { recordTestResult } from "../lib/state";
import { getCurrentTimestamp } from "../lib/utils";
import type { TestResult } from "../lib/types";

export async function handleRun(args: string[]): Promise<void> {
  if (args.length === 0) {
    console.error("Usage: tcr run <files...>");
    console.error("Example: tcr run src/App.tsx src/utils/auth.ts");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();

  // Find test files for the given source files
  const testFiles = await findTestFiles(args, projectRoot);

  if (testFiles.length === 0) {
    console.log("No test files found for the specified source files.");
    console.log("Convention: foo.ts → foo.test.ts or foo.spec.ts");
    return;
  }

  console.log(`Found ${testFiles.length} test file(s):`);
  for (const file of testFiles) {
    console.log(`  - ${file}`);
  }
  console.log("");

  // Determine target based on source files
  const target = determineTarget(args);
  console.log(`Running ${target} tests...`);
  console.log("");

  // Run tests
  const result = await runTests(target, testFiles, projectRoot);

  // Display output
  console.log(formatTestOutput(result));

  // Record result
  const testResult: TestResult = {
    passed: result.passed,
    timestamp: getCurrentTimestamp(),
    error: result.error,
    filesRun: testFiles,
    target,
  };
  await recordTestResult(projectRoot, testResult);

  // Exit with appropriate code
  if (result.passed) {
    console.log("\n✅ All tests passed");
  } else {
    console.log("\n❌ Tests failed");
    process.exit(1);
  }
}
