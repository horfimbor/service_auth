build:
	docker build --build-arg MOD_AUTH_SECRET_DEFAULT="dev" -t horfimbor/service_auth:dev .