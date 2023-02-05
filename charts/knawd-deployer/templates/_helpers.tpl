{{/*
Expand the name of the chart.
*/}}
{{- define "knawd-deployer.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "knawd-deployer.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "knawd-deployer.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "knawd-deployer.labels" -}}
helm.sh/chart: {{ include "knawd-deployer.chart" . }}
{{ include "knawd-deployer.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "knawd-deployer.selectorLabels" -}}
app.kubernetes.io/name: {{ include "knawd-deployer.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "knawd-deployer.serviceAccountName" -}}
"knawd-deployer-sa"
{{- end }}

{{/*
Create the name of the Security Context Constraint to use
*/}}
{{- define "knawd-deployer.sccName" -}}
"knawd-deployer-scc"
{{- end }}

{{/*
Create the role name to use
*/}}
{{- define "knawd-deployer.roleName" -}}
"knawd-deployer-role"
{{- end }}

{{/*
Calculate the location for the library
*/}}
{{- define "knawd-deployer.libLocation" -}}
{{- if eq .Values.target "rhel8" }}"/usr/lib64"{{- else }}"/lib"{{- end }}
{{- end }}

{{/*
Calculate the OCI Binary
*/}}
{{- define "knawd-deployer.ociLocation" -}}
"/usr/local/sbin"
{{- end }}

{{/*
Calculate the location to mount the host route in the container
*/}}
{{- define "knawd-deployer.nodeRoot" -}}
"/mnt/node-root"
{{- end }}

{{/*
Calculate the location for configuration
*/}}
{{- define "knawd-deployer.configLocation" -}}
{{- if eq .Values.target "rhel8" }}"/etc/crio"{{- else }}"/etc/containerd"{{- end}}
{{- end }}

{{- define "knawd-deployer.isMicroK8s" -}}
{{- if eq .Values.target "microk8s" }}"true"{{- else }}"false"{{- end}}
{{- end }}


