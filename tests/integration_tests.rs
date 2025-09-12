// Integration test runner for end-to-end scenarios
// This file allows running tests from subdirectories

mod integration {
    mod test_new_project_setup;
    mod test_existing_project_install;
    mod test_script_execution;
    mod test_dependency_management;
    mod test_offline_development;
    mod test_npm_integration;
    mod test_pypi_integration;
    mod test_package_installer;
    mod test_symlink_manager;
}
