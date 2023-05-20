import jinja2
import os
from base64 import b64encode
import sys
from os.path import exists


if len(sys.argv) != 2:
    raise Exception("Command line only expects one argument(base template file path)")


if not exists(sys.argv[1]):
    raise Exception("Specified template file doesnt exist")

template_values = [var for var in os.environ if var.find("APP_") != -1]
env_values = [var for var in os.environ if var.find("ENV_") != -1]

config = {}
env_variables = {}

for value in template_values:
    config[value.lower().replace("app_", "")] = os.environ[value]

for variable in env_values:
    encoded_val = b64encode(os.environ[variable].encode("ascii")).decode("ascii")
    env_variables[variable.lower().replace("env_", "")] = encoded_val

templateLoader = jinja2.FileSystemLoader(searchpath="./")
templateEnv = jinja2.Environment(loader=templateLoader)

template = templateEnv.get_template(sys.argv[1])

print(template.render(config=config, variables=env_variables))
