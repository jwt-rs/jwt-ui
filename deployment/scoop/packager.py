#import hashlib
import sys
from string import Template

args = sys.argv
version64 = args[1].replace("v", "")
template_file_path = args[2]
generated_file_path = args[3]

# Deployment files
hash_64 = args[4].strip()

print("Generating formula")
print(f"     VERSION: {version64}")
print(f"     TEMPLATE PATH: {template_file_path}")
print(f"     SAVING AT: {generated_file_path}")
print(f"     HASH: {hash_64}")

with open(template_file_path, "r") as template_file:
    template = Template(template_file.read())
    substitute = template.safe_substitute(version64=version64, hash_64=hash_64)
    print("\n================== Generated package file ==================\n")
    print(substitute)
    print("\n============================================================\n")

    with open(generated_file_path, "w") as generated_file:
        generated_file.write(substitute)