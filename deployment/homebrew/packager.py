import hashlib
import sys
from string import Template

args = sys.argv
version = args[1]
template_file_path = args[2]
generated_file_path = args[3]

# Deployment files
hash_mac = args[4].strip()
hash_mac_arm = args[5].strip()
hash_linux = args[6].strip()

print("Generating formula")
print(f"     VERSION: {version}")
print(f"     TEMPLATE PATH: {template_file_path}")
print(f"     SAVING AT: {generated_file_path}")
print(f"     MAC HASH: {hash_mac}")
print(f"     MAC ARM HASH: {hash_mac_arm}")
print(f"     LINUX HASH: {hash_linux}")

with open(template_file_path, "r") as template_file:
    template = Template(template_file.read())
    substitute = template.safe_substitute(version=version, hash_mac=hash_mac, hash_mac_arm=hash_mac_arm, hash_linux=hash_linux)
    print("\n================== Generated package file ==================\n")
    print(substitute)
    print("\n============================================================\n")

    with open(generated_file_path, "w") as generated_file:
        generated_file.write(substitute)