#!/usr/bin/sh

uvicorn caoloweb.app:app --host 0.0.0.0 --port ${PORT:-8000} --workers ${WEB_CONCURRENCY:-8}
